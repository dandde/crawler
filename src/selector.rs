use select::node::Node;
use select::predicate::Predicate;
use serde::{Deserialize, Serialize};

/// A recursive, serializable definition of a CSS selector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "spec")]
pub enum CssSelector {
    /// Matches an HTML tag name (e.g., "div", "a")
    Tag(String),

    /// Matches a CSS class (e.g., "quote")
    Class(String),

    /// Matches an HTML ID (e.g., "main")
    Id(String),

    /// Matches an attribute existence or specific value
    Attribute { key: String, value: Option<String> },

    /// AND Logic: Matches if ALL sub-selectors match
    And(Vec<CssSelector>),

    /// OR Logic: Matches if ANY sub-selector matches
    Or(Vec<CssSelector>),

    /// Descendant Logic: .ancestor .descendant
    Descendant {
        ancestor: Box<CssSelector>,
        descendant: Box<CssSelector>,
    },

    /// Child Logic: .parent > .child
    Child {
        parent: Box<CssSelector>,
        child: Box<CssSelector>,
    },
}

impl CssSelector {
    /// Converts the structured selector into a standard CSS selector string.
    pub fn to_css_string(&self) -> String {
        match self {
            CssSelector::Tag(tag) => tag.clone(),
            CssSelector::Class(cls) => format!(".{}", cls),
            CssSelector::Id(id) => format!("#{}", id),
            CssSelector::Attribute { key, value } => match value {
                Some(v) => format!("[{}='{}']", key, v),
                None => format!("[{}]", key),
            },
            CssSelector::And(selectors) => selectors
                .iter()
                .map(|s| s.to_css_string())
                .collect::<Vec<_>>()
                .join(""),
            CssSelector::Or(selectors) => selectors
                .iter()
                .map(|s| s.to_css_string())
                .collect::<Vec<_>>()
                .join(", "),
            CssSelector::Descendant {
                ancestor,
                descendant,
            } => {
                format!(
                    "{} {}",
                    ancestor.to_css_string(),
                    descendant.to_css_string()
                )
            }
            CssSelector::Child { parent, child } => {
                format!("{} > {}", parent.to_css_string(), child.to_css_string())
            }
        }
    }
}

impl Predicate for CssSelector {
    fn matches(&self, node: &Node) -> bool {
        match self {
            CssSelector::Tag(tag) => node.name() == Some(tag),
            CssSelector::Class(cls) => node
                .attr("class")
                .map(|classes| classes.split_whitespace().any(|c| c == cls))
                .unwrap_or(false),
            CssSelector::Id(id) => node.attr("id") == Some(id),
            CssSelector::Attribute { key, value } => match value {
                Some(v) => node.attr(key.as_str()) == Some(v),
                None => node.attr(key.as_str()).is_some(),
            },
            CssSelector::And(selectors) => selectors.iter().all(|s| s.matches(node)),
            CssSelector::Or(selectors) => selectors.iter().any(|s| s.matches(node)),
            CssSelector::Descendant {
                ancestor,
                descendant,
            } => {
                if !descendant.matches(node) {
                    return false;
                }
                let mut current = node.parent();
                while let Some(parent) = current {
                    if ancestor.matches(&parent) {
                        return true;
                    }
                    current = parent.parent();
                }
                false
            }
            CssSelector::Child { parent, child } => {
                child.matches(node) && node.parent().map(|p| parent.matches(&p)).unwrap_or(false)
            }
        }
    }
}

impl<'a> Predicate for &'a CssSelector {
    fn matches(&self, node: &Node) -> bool {
        (*self).matches(node)
    }
}
