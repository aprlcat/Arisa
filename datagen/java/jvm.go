package main

import (
	"bytes"
	"crypto/tls"
	"encoding/json"
	"fmt"
	"html"
	"io/ioutil"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/PuerkitoBio/goquery"
	"github.com/charmbracelet/log"
)

const (
	sourceURL      = "https://en.wikipedia.org/wiki/List_of_Java_bytecode_instructions"
	outputFilename = "jvm_instructions.json"
	requestTimeout = 30 * time.Second
)

type InstructionData struct {
	Mnemonic     string `json:"mnemonic"`
	OpcodeHex    string `json:"opcodeHex"`
	OpcodeBinary string `json:"opcodeBinary"`
	OtherBytes   string `json:"otherBytes"`
	Stack        string `json:"stack"`
	Description  string `json:"description"`
}

type Scraper struct {
	client *http.Client
	logger *log.Logger
}

func NewScraper() *Scraper {
	logger := log.NewWithOptions(os.Stderr, log.Options{
		ReportCaller:    false,
		ReportTimestamp: true,
		TimeFormat:      time.Kitchen,
		Prefix:          "jvm-scraper",
	})

	client := &http.Client{
		Timeout: requestTimeout,
		Transport: &http.Transport{
			TLSClientConfig:   &tls.Config{InsecureSkipVerify: false},
			DisableKeepAlives: false,
			MaxIdleConns:      10,
			IdleConnTimeout:   90 * time.Second,
		},
	}

	return &Scraper{
		client: client,
		logger: logger,
	}
}

func (s *Scraper) cleanText(text string) string {
	text = html.UnescapeString(text)
	text = regexp.MustCompile(`\s+`).ReplaceAllString(text, " ")
	text = strings.TrimSpace(text)
	return text
}

func (s *Scraper) fetchPage() (*goquery.Document, error) {
	s.logger.Info("Fetching instruction data")

	req, err := http.NewRequest("GET", sourceURL, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("User-Agent", "jvm-scraper/1.0")

	resp, err := s.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch URL: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("bad status: %s", resp.Status)
	}

	doc, err := goquery.NewDocumentFromReader(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to parse HTML: %w", err)
	}

	return doc, nil
}

func (s *Scraper) parseInstructionTable(doc *goquery.Document) []InstructionData {
	var instructions []InstructionData

	doc.Find("table.wikitable tbody tr").Each(func(i int, row *goquery.Selection) {
		cells := row.Find("td")
		if cells.Length() < 6 {
			return
		}

		instruction := InstructionData{
			Mnemonic:     s.cleanText(cells.Eq(0).Text()),
			OpcodeHex:    s.cleanText(cells.Eq(1).Text()),
			OpcodeBinary: s.cleanText(cells.Eq(2).Text()),
			OtherBytes:   s.cleanText(cells.Eq(3).Text()),
			Stack:        s.cleanText(cells.Eq(4).Text()),
			Description:  s.cleanText(cells.Eq(5).Text()),
		}

		if instruction.Mnemonic != "" {
			instructions = append(instructions, instruction)
		}
	})

	return instructions
}

func (s *Scraper) convertToJVMFormat(instructions []InstructionData) []map[string]interface{} {
	var jvmInstructions []map[string]interface{}

	for _, inst := range instructions {
		jvmInst := make(map[string]interface{})

		jvmInst["mnemonic"] = inst.Mnemonic

		opcodeValue := s.extractOpcodeValue(inst.OpcodeHex)
		if opcodeValue != "" {
			jvmInst["opcode"] = fmt.Sprintf("%s = %s (0x%s)", inst.Mnemonic, opcodeValue, inst.OpcodeHex)
		}

		jvmInst["operation"] = inst.Description

		format := inst.Mnemonic
		if inst.OtherBytes != "" {
			format += " " + strings.ReplaceAll(inst.OtherBytes, ":", "")
		}
		jvmInst["format"] = format

		before, after := s.parseStack(inst.Stack)
		jvmInst["operandStackBefore"] = before
		jvmInst["operandStackAfter"] = after

		jvmInst["description"] = inst.Description

		jvmInst["anchorId"] = "jvm-" + strings.ReplaceAll(inst.Mnemonic, "_", "-")

		jvmInstructions = append(jvmInstructions, jvmInst)
	}

	return jvmInstructions
}

func (s *Scraper) extractOpcodeValue(hex string) string {
	if hex == "" {
		return ""
	}

	hexValue := strings.TrimSpace(hex)
	if val, err := strconv.ParseInt(hexValue, 16, 64); err == nil {
		return strconv.FormatInt(val, 10)
	}

	return ""
}

func (s *Scraper) parseStack(stack string) (string, string) {
	if stack == "" || stack == "[No change]" || stack == "[no change]" {
		return "No change", "No change"
	}

	if strings.Contains(stack, "→") {
		parts := strings.Split(stack, "→")
		if len(parts) == 2 {
			before := strings.TrimSpace(parts[0])
			after := strings.TrimSpace(parts[1])

			if before == "" {
				before = "..."
			}
			if after == "" || after == "[empty]" {
				after = "[empty]"
			}

			return before, after
		}
	}

	return "...", stack
}

func (s *Scraper) scrapeInstructions() ([]map[string]interface{}, error) {
	doc, err := s.fetchPage()
	if err != nil {
		return nil, err
	}

	instructions := s.parseInstructionTable(doc)
	s.logger.Info("Parsed instructions", "count", len(instructions))

	jvmInstructions := s.convertToJVMFormat(instructions)

	return jvmInstructions, nil
}

func (s *Scraper) saveData(instructions []map[string]interface{}) error {
	s.logger.Info("Saving instruction data", "count", len(instructions))

	buffer := new(bytes.Buffer)
	encoder := json.NewEncoder(buffer)
	encoder.SetEscapeHTML(false)
	encoder.SetIndent("", "  ")

	if err := encoder.Encode(instructions); err != nil {
		return fmt.Errorf("failed to encode JSON: %w", err)
	}

	if err := ioutil.WriteFile(outputFilename, buffer.Bytes(), 0644); err != nil {
		return fmt.Errorf("failed to write JSON to file: %w", err)
	}

	s.logger.Info("Data saved successfully", "file", outputFilename)
	return nil
}

func (s *Scraper) Run() error {
	s.logger.Info("Starting JVM instruction scraper")

	instructions, err := s.scrapeInstructions()
	if err != nil {
		return fmt.Errorf("failed to scrape instructions: %w", err)
	}

	if err := s.saveData(instructions); err != nil {
		return fmt.Errorf("failed to save data: %w", err)
	}

	s.logger.Info("Scraping completed successfully")
	return nil
}

func main() {
	scraper := NewScraper()
	if err := scraper.Run(); err != nil {
		scraper.logger.Fatal("Scraper failed", "error", err)
	}
}
