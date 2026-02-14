# Crawler Configuration Guide

The Unified Crawler uses a flexible, multi-format configuration system. You can define your scraping jobs in **JSON**, **YAML**, or **TOML**.

---

## 🏗️ Configuration Structure

A spider configuration file consists of the following top-level fields:

| Field | Type | Description | Required |
|:--- |:--- |:--- |:--- |
| `name` | String | Unique identifier for the crawl session. | Yes |
| `start_urls` | Array | List of URLs to begin crawling from. | Yes |
| `root_selector` | Selector | Selector for identifying individual items on a page. | No |
| `extraction_rules` | Map | Key-value pairs of field names and their extraction rules. | Yes |
| `output` | Object | Configuration for data persistence (Console, JSON, CSV, SQLite). | No |
| `concurrency` | Integer | Number of concurrent requests (default: 2). | No |
| `delay_ms` | Integer | Delay between requests in milliseconds (default: 500). | No |
| `extends` | Path | Relative path to a parent config for inheritance. | No |

---

## 🎯 Selector System (Two Variants)

The crawler supports two ways to define selectors. Choose the one that fits your complexity level.

### 1. Simple String Selectors (Recommended)
You can use standard selector strings with an optional engine prefix. If no prefix is provided, `css:` is assumed.

- **CSS Selectors**: `css:.quote.text` or simply `.quote.text`
- **XPath Selectors**: `xpath://div[@class='quote']`
- **Regex Patterns**: `regex:author: (.*)`

### 2. Advanced Structured Selectors
For complex logic, you can use recursive objects. This matches the [Technical Reference](file:///Volumes/Mac_Data/Mac_mini/development/rust/crawler_workspace/crawl-cli/selector_config_help.html).

**Key Variants:**
- `Tag`: Match by HTML tag name.
- `Class`: Match by CSS class.
- `Id`: Match by HTML ID.
- `Attribute`: Match by attribute existence or specific value.
- `And`/`Or`: Logical combinations of selectors.
- `Descendant`/`Child`: Positional relationships.

---

## 🔄 Selector Translation Guide

Convert your standard selectors into the crawler format using this table:

| Standard Format | Standard Example | Crawler String Format | Crawler Advanced Object (JSON Example) |
|:--- |:--- |:--- |:--- |
| **CSS Tag** | `div` | `"css:div"` | `{"kind": "Tag", "spec": "div"}` |
| **CSS Class** | `.quote` | `"css:.quote"` | `{"kind": "Class", "spec": "quote"}` |
| **CSS ID** | `#main` | `"css:#main"` | `{"kind": "Id", "spec": "main"}` |
| **CSS Combined** | `span.text` | `"css:span.text"` | `{"kind": "And", "spec": [{"kind": "Tag", "spec": "span"}, {"kind": "Class", "spec": "text"}]}` |
| **CSS Child** | `.tags > a` | `"css:.tags > a"` | `{"kind": "Child", "spec": {"parent": {"kind": "Class", "spec": "tags"}, "child": {"kind": "Tag", "spec": "a"}}}` |
| **XPath** | `//a[@href]` | `"xpath://a[@href]"` | *Not supported in advanced object mode* |

---

## 📝 Extraction Rules

Each rule defines **where** to look (selector) and **what** to take (extract).

```json
"extraction_rules": {
  "content": {
    "selector": "css:.text",
    "extract": "text"
  },
  "link": {
    "selector": "css:a",
    "extract": { "attribute": "href" }
  },
  "raw": {
    "selector": "css:div.info",
    "extract": "html"
  }
}
```

---

## 🚀 Full Examples

````carousel
```json
{
  "name": "quotes-json",
  "start_urls": ["https://quotes.toscrape.com/"],
  "root_selector": ".quote",
  "extraction_rules": {
    "author": { "selector": ".author", "extract": "text" },
    "quote": { "selector": ".text", "extract": "text" }
  },
  "output": { "type": "json", "path": "outputs/quotes.json" }
}
```
<!-- slide -->
```yaml
name: quotes-yaml
start_urls:
  - https://quotes.toscrape.com/
root_selector: 
  kind: Class
  spec: quote
extraction_rules:
  author:
    selector: 
       kind: Tag
       spec: small
    extract: text
  tags:
    selector: .tags
    extract: html
output:
  type: csv
  path: outputs/quotes.csv
```
<!-- slide -->
```toml
name = "quotes-toml"
start_urls = ["https://quotes.toscrape.com/"]
root_selector = ".quote"

[extraction_rules.author]
selector = "xpath:.//small[@class='author']/text()"
extract = "text"

[output]
type = "sqlite"
path = "outputs/quotes.db"
table = "quotes"
```
````

---

## 🏗️ Configuration Inheritance

Reuse shared logic using the `extends` field.

**base.toml**
```toml
concurrency = 5
delay_ms = 1000
[extraction_rules.site_version]
selector = "meta[name='version']"
extract = { attribute = "content" }
```

**derived.toml**
```toml
extends = "base.toml"
name = "my-spider"
start_urls = ["https://example.com"]
```

The `derived.toml` will inherit the concurrency, delay, and extraction rules from `base.toml`.
