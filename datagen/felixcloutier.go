package main

import (
	"bytes"
	"crypto/tls"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/PuerkitoBio/goquery"
	"github.com/charmbracelet/log"
)

const (
	baseURL        = "https://www.felixcloutier.com"
	indexURL       = baseURL + "/x86/"
	outputFilename = "x86.json"
	numWorkers     = 50
	requestTimeout = 15 * time.Second
)

type TableRow map[string]string

type InstructionData struct {
	URL                  string              `json:"url"`
	Category             string              `json:"category"`
	InstructionName      string              `json:"instructionName"`
	DetailsTable         []TableRow          `json:"detailsTable"`
	OperandEncodingTable []TableRow          `json:"operandEncodingTable"`
	DescriptionText      string              `json:"descriptionText"`
	OperationText        string              `json:"operationText"`
	FlagsAffectedText    string              `json:"flagsAffectedText"`
	Exceptions           map[string][]string `json:"exceptions"`
	Error                string              `json:"error,omitempty"`
}

type InstructionLink struct {
	URL      string
	Category string
}

type Scraper struct {
	client              *http.Client
	logger              *log.Logger
	previousData        map[string]InstructionData
	successfullyScraped map[string]bool
}

func NewScraper() *Scraper {
	logger := log.NewWithOptions(os.Stderr, log.Options{
		ReportCaller:    false,
		ReportTimestamp: true,
		TimeFormat:      time.Kitchen,
		Prefix:          "x86-scraper",
	})

	client := &http.Client{
		Timeout: requestTimeout,
		Transport: &http.Transport{
			TLSClientConfig:   &tls.Config{InsecureSkipVerify: false},
			DisableKeepAlives: false,
			MaxIdleConns:      100,
			IdleConnTimeout:   90 * time.Second,
		},
	}

	return &Scraper{
		client:              client,
		logger:              logger,
		previousData:        make(map[string]InstructionData),
		successfullyScraped: make(map[string]bool),
	}
}

func (s *Scraper) loadExistingData() error {
	if _, err := os.Stat(outputFilename); os.IsNotExist(err) {
		s.logger.Info("No existing data file found, starting fresh")
		return nil
	}

	s.logger.Info("Loading existing data", "file", outputFilename)

	fileBytes, err := ioutil.ReadFile(outputFilename)
	if err != nil {
		s.logger.Warn("Could not read existing data file", "error", err)
		return err
	}

	var loadedData []InstructionData
	if err := json.Unmarshal(fileBytes, &loadedData); err != nil {
		s.logger.Warn("Could not unmarshal existing data", "error", err)
		return err
	}

	for _, item := range loadedData {
		s.previousData[item.URL] = item
		if item.Error == "" {
			s.successfullyScraped[item.URL] = true
		}
	}

	s.logger.Info("Loaded previous data",
		"total_entries", len(s.previousData),
		"successful", len(s.successfullyScraped))

	return nil
}

func (s *Scraper) parseTableFromGoquery(tableSelection *goquery.Selection) []TableRow {
	var tableData []TableRow
	var headers []string

	tableSelection.Find("tr").Each(func(i int, rowSelection *goquery.Selection) {
		if i == 0 {
			rowSelection.Find("th").Each(func(_ int, thSelection *goquery.Selection) {
				headers = append(headers, strings.TrimSpace(thSelection.Text()))
			})
		} else {
			rowData := make(TableRow)
			rowSelection.Find("td").Each(func(j int, tdSelection *goquery.Selection) {
				headerKey := fmt.Sprintf("column_%d", j+1)
				if j < len(headers) && headers[j] != "" {
					headerKey = headers[j]
				}
				rowData[headerKey] = strings.TrimSpace(tdSelection.Text())
			})
			if len(rowData) > 0 {
				tableData = append(tableData, rowData)
			}
		}
	})
	return tableData
}

func (s *Scraper) extractTextFollowingHeader(doc *goquery.Document, headerID string) string {
	var content []string
	header := doc.Find(fmt.Sprintf("h2#%s", headerID))
	if header.Length() > 0 {
		currentNode := header.Next()
		for currentNode.Length() > 0 && currentNode.Get(0).Data != "h2" {
			if currentNode.Is("p") || currentNode.Is("pre") {
				content = append(content, strings.TrimSpace(currentNode.Text()))
			}
			currentNode = currentNode.Next()
		}
	}
	return strings.Join(content, "\n")
}

func (s *Scraper) parseInstructionPage(pageURL, category string) InstructionData {
	data := InstructionData{
		URL:      pageURL,
		Category: category,
	}

	req, err := http.NewRequest("GET", pageURL, nil)
	if err != nil {
		data.Error = fmt.Sprintf("failed to create request: %v", err)
		return data
	}
	req.Header.Set("User-Agent", "x86-scraper/1.0 (+https://github.com/user/x86-scraper)")

	resp, err := s.client.Do(req)
	if err != nil {
		data.Error = fmt.Sprintf("failed to fetch URL: %v", err)
		return data
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		data.Error = fmt.Sprintf("bad status: %s", resp.Status)
		return data
	}

	doc, err := goquery.NewDocumentFromReader(resp.Body)
	if err != nil {
		data.Error = fmt.Sprintf("failed to parse HTML: %v", err)
		return data
	}

	data.InstructionName = strings.TrimSpace(doc.Find("h1").First().Text())

	allTables := doc.Find("table")
	if allTables.Length() > 0 {
		data.DetailsTable = s.parseTableFromGoquery(allTables.First())

		operandEncodingHeader := doc.Find("h2#instruction-operand-encoding")
		if operandEncodingHeader.Length() > 0 {
			operandEncodingTableElement := operandEncodingHeader.NextFiltered("table")
			data.OperandEncodingTable = s.parseTableFromGoquery(operandEncodingTableElement)
		} else if allTables.Length() > 1 {
			possibleOperandTable := allTables.Eq(1)
			isOperandTable := false
			possibleOperandTable.Find("th").Each(func(_ int, th *goquery.Selection) {
				if strings.TrimSpace(th.Text()) == "Op/En" {
					isOperandTable = true
				}
			})
			if isOperandTable {
				data.OperandEncodingTable = s.parseTableFromGoquery(possibleOperandTable)
			}
		}
	}

	data.DescriptionText = s.extractTextFollowingHeader(doc, "description")
	data.OperationText = s.extractTextFollowingHeader(doc, "operation")
	data.FlagsAffectedText = s.extractTextFollowingHeader(doc, "flags-affected")

	data.Exceptions = make(map[string][]string)
	doc.Find("h2.exceptions").Each(func(_ int, exceptionHeader *goquery.Selection) {
		modeName := s.parseExceptionModeName(exceptionHeader.Text())

		var exceptionContent []string
		currentNode := exceptionHeader.Next()
		for currentNode.Length() > 0 && currentNode.Get(0).Data != "h2" {
			if currentNode.Is("p") {
				exceptionContent = append(exceptionContent, strings.TrimSpace(currentNode.Text()))
			} else if currentNode.Is("table") {
				var tableText strings.Builder
				parsedTable := s.parseTableFromGoquery(currentNode)
				for _, tr := range parsedTable {
					for k, v := range tr {
						tableText.WriteString(fmt.Sprintf("%s: %s; ", k, v))
					}
					tableText.WriteString("\n")
				}
				exceptionContent = append(exceptionContent, strings.TrimSpace(tableText.String()))
			}
			currentNode = currentNode.Next()
		}
		data.Exceptions[modeName] = exceptionContent
	})

	return data
}

func (s *Scraper) parseExceptionModeName(text string) string {
	text = strings.TrimSpace(text)

	modeMap := map[string]string{
		"64-Bit Mode":        "64BitMode",
		"Protected Mode":     "protectedMode",
		"Real-Address Mode":  "realAddressMode",
		"Virtual-8086 Mode":  "virtual8086Mode",
		"Compatibility Mode": "compatibilityMode",
	}

	for key, value := range modeMap {
		if strings.Contains(text, key) {
			return value
		}
	}

	parts := strings.Fields(strings.ReplaceAll(text, " Exceptions", ""))
	if len(parts) > 0 {
		modeName := strings.ToLower(parts[0])
		for _, p := range parts[1:] {
			modeName += strings.Title(strings.ToLower(p))
		}
		return modeName
	}

	return "unknownMode"
}

func (s *Scraper) fetchInstructionLinks() ([]InstructionLink, error) {
	s.logger.Info("Fetching instruction links from index page")

	resp, err := s.client.Get(indexURL)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch index page: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("bad status for index page: %s", resp.Status)
	}

	doc, err := goquery.NewDocumentFromReader(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to parse index page HTML: %w", err)
	}

	var linksToScrape []InstructionLink
	processedURLs := make(map[string]bool)

	doc.Find("h2").Each(func(_ int, h2Selection *goquery.Selection) {
		categoryName := strings.TrimSpace(h2Selection.Text())

		if s.isInstructionSection(categoryName) {
			table := h2Selection.NextFiltered("table")
			table.Find("tr td:first-child a").Each(func(_ int, linkSelection *goquery.Selection) {
				href, exists := linkSelection.Attr("href")
				if !exists {
					return
				}

				fullURL := s.resolveURL(href)
				if fullURL == "" {
					return
				}

				if !processedURLs[fullURL] {
					if _, previouslySuccessful := s.successfullyScraped[fullURL]; !previouslySuccessful {
						linksToScrape = append(linksToScrape, InstructionLink{
							URL:      fullURL,
							Category: categoryName,
						})
					}
					processedURLs[fullURL] = true
				}
			})
		}
	})

	s.logger.Info("Found instruction links",
		"total_on_index", len(processedURLs),
		"to_scrape", len(linksToScrape))

	return linksToScrape, nil
}

func (s *Scraper) isInstructionSection(categoryName string) bool {
	instructionSections := []string{
		"Core Instructions",
		"SGX Instructions",
		"SMX Instructions",
		"VMX Instructions",
		"Xeon Phi™ Instructions",
	}

	for _, section := range instructionSections {
		if categoryName == section {
			return true
		}
	}

	return strings.Contains(strings.ToLower(categoryName), "instructions")
}

func (s *Scraper) resolveURL(href string) string {
	if strings.HasPrefix(href, "/x86/") {
		return baseURL + href
	}

	if strings.HasPrefix(href, "http") {
		return href
	}

	tempURL, err := url.Parse(indexURL)
	if err != nil {
		s.logger.Error("Error parsing indexURL for relative path resolution", "error", err)
		return ""
	}

	relURL, err := url.Parse(href)
	if err != nil {
		s.logger.Error("Error parsing relative href", "href", href, "error", err)
		return ""
	}

	fullURL := tempURL.ResolveReference(relURL).String()

	if u, err := url.Parse(fullURL); err == nil {
		return u.String()
	}

	s.logger.Warn("Error normalizing URL", "url", fullURL)
	return fullURL
}

func (s *Scraper) scrapeInstructions(links []InstructionLink) map[string]InstructionData {
	if len(links) == 0 {
		s.logger.Info("No new or failed URLs to scrape")
		return make(map[string]InstructionData)
	}

	workers := numWorkers
	if len(links) < workers {
		workers = len(links)
	}

	s.logger.Info("Starting concurrent scraping",
		"workers", workers,
		"total_links", len(links))

	jobs := make(chan InstructionLink, len(links))
	results := make(chan InstructionData, len(links))
	var wg sync.WaitGroup

	for i := 0; i < workers; i++ {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()
			for link := range jobs {
				s.logger.Debug("Scraping instruction",
					"worker", workerID,
					"url", link.URL)

				result := s.parseInstructionPage(link.URL, link.Category)
				results <- result
			}
		}(i)
	}

	for _, link := range links {
		jobs <- link
	}
	close(jobs)

	go func() {
		wg.Wait()
		close(results)
	}()

	scrapedData := make(map[string]InstructionData)
	errorCount := 0

	for result := range results {
		if result.Error != "" {
			s.logger.Error("Error scraping instruction",
				"url", result.URL,
				"error", result.Error)
			errorCount++
		} else {
			s.logger.Debug("Successfully scraped instruction",
				"url", result.URL,
				"name", result.InstructionName)
		}
		scrapedData[result.URL] = result
	}

	s.logger.Info("Scraping completed",
		"scraped", len(scrapedData),
		"errors", errorCount)

	return scrapedData
}

func (s *Scraper) saveData(currentData map[string]InstructionData) error {
	s.logger.Info("Preparing final dataset")

	finalData := make(map[string]InstructionData)
	for url, data := range s.previousData {
		finalData[url] = data
	}
	for url, data := range currentData {
		finalData[url] = data
	}

	var finalSlice []InstructionData
	for _, data := range finalData {
		finalSlice = append(finalSlice, data)
	}

	s.logger.Info("Final dataset prepared", "total_instructions", len(finalSlice))

	buffer := new(bytes.Buffer)
	encoder := json.NewEncoder(buffer)
	encoder.SetEscapeHTML(false)
	encoder.SetIndent("", "  ")

	if err := encoder.Encode(finalSlice); err != nil {
		return fmt.Errorf("failed to encode JSON: %w", err)
	}

	if err := ioutil.WriteFile(outputFilename, buffer.Bytes(), 0644); err != nil {
		return fmt.Errorf("failed to write JSON to file: %w", err)
	}

	s.logger.Info("Data saved successfully", "file", outputFilename)

	errorCount := 0
	for _, inst := range finalSlice {
		if inst.Error != "" {
			errorCount++
		}
	}

	if errorCount > 0 {
		s.logger.Warn("Dataset contains errors", "error_count", errorCount)
	}

	return nil
}

func (s *Scraper) Run() error {
	s.logger.Info("Starting x86 instruction scraper")

	if err := s.loadExistingData(); err != nil {
		s.logger.Warn("Failed to load existing data, continuing with fresh start", "error", err)
	}

	links, err := s.fetchInstructionLinks()
	if err != nil {
		return fmt.Errorf("failed to fetch instruction links: %w", err)
	}

	currentData := s.scrapeInstructions(links)

	if err := s.saveData(currentData); err != nil {
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
