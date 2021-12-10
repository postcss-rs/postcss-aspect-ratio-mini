use std::{borrow::Cow, io::Write, ops::AddAssign};

use recursive_parser::{parser::*, visitor::VisitMut, WrapString};
use regex::{Captures, Regex};

#[derive(Default)]
pub struct AspectRatioMini {}

impl AspectRatioMini {
    pub fn transform(root: &mut Root, indent: usize) -> String {
        let mut aspect = AspectRatioMini::default();
        aspect.visit_root(root);
        let mut printer = SimplePrettier::new(WrapString::default(), indent);
        printer.visit_root(root).unwrap();
        printer.writer.0
    }
}
impl<'a> VisitMut<'a, bool> for AspectRatioMini {
    fn visit_root(&mut self, root: &mut Root<'a>) -> bool {
        root.children.iter_mut().for_each(|child| match child {
            RuleOrAtRuleOrDecl::Rule(rule) => {
                self.visit_rule(rule);
            }
            RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                self.visit_at_rule(at_rule);
            }
            RuleOrAtRuleOrDecl::Declaration(_) => {
                unreachable!()
            }
        });
        false
    }
    // TODO: add :before to every selector, waiting for selector-parser
    fn visit_rule(&mut self, rule: &mut Rule<'a>) -> bool {
        let mut has_ratio_prop = false;
        rule.children
            .iter_mut()
            .for_each(|rule_child| match rule_child {
                RuleOrAtRuleOrDecl::Rule(_) => {
                    unreachable!()
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule);
                }
                RuleOrAtRuleOrDecl::Declaration(decl) => {
                    has_ratio_prop |= self.visit_declaration(decl);
                }
            });
        if has_ratio_prop {
            rule.selector.content.add_assign(":before");
        }
        false
    }
    fn visit_at_rule(&mut self, at_rule: &mut AtRule<'a>) -> bool {
        at_rule
            .children
            .iter_mut()
            .for_each(|rule_child| match rule_child {
                RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule);
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule);
                }
                RuleOrAtRuleOrDecl::Declaration(_decl) => {
                    unreachable!()
                }
            });
        false
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) -> bool {
        // decl.prop.content = Cow::Owned(decl.prop.content.chars().rev().collect());
        let need_process = match &decl.prop.content {
            Cow::Borrowed(content) => {
                matches!(content, &"aspect-ratio" | &"aspect" | &"ratio")
            }
            Cow::Owned(content) => {
                matches!(content.as_str(), "aspect-ratio" | "aspect" | "ratio")
            }
        };
        if !need_process {
            return false;
        }

        let value = process_ration_value(&decl.value.content);
        process_ratio_conf(decl, value);
        true
    }
}

fn process_ration_value(decl: &str) -> String {
    let re = Regex::new(r#"['"]?(?:((?:\d*\.?\d*)?)(?:\s*[/]\s*)(\d*\.?\d*))['"]?"#).unwrap();
    re.replace_all(decl, |caps: &Captures| {
        let computed_result = caps[2]
            .parse::<f32>()
            .and_then(|y| caps[1].parse::<f32>().map(|x| y / x * 100f32));
        match computed_result {
            Ok(value) => value.to_string() + "%",
            Err(_) => decl.to_string(),
        }
    })
    .to_string()
}

fn process_ratio_conf(decl: &mut Declaration, ratio: String) {
    decl.prop.content = Cow::Owned(("padding-top").to_string());
    decl.value.content = Cow::Owned(ratio);
}

#[derive(Default)]
pub struct SimplePrettier<W: Write> {
    level: usize,
    pub writer: W,
    indent: usize,
}

impl<W: Write> SimplePrettier<W> {
    pub fn new(writer: W, indent: usize) -> Self {
        Self {
            level: 0,
            writer,
            indent,
        }
    }
}
impl<'a, W: std::io::Write> VisitMut<'a, std::io::Result<()>> for SimplePrettier<W> {
    fn visit_root(&mut self, root: &mut Root<'a>) -> std::io::Result<()> {
        for child in root.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule)?;
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(_) => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }

    fn visit_rule(&mut self, rule: &mut Rule<'a>) -> std::io::Result<()> {
        self.writer.write_all(
            format!(
                "{}{} {}\n",
                " ".repeat(self.level * self.indent),
                rule.selector.content,
                "{"
            )
            .as_bytes(),
        )?;
        self.level += 1;
        for child in rule.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(_) => {
                    unreachable!()
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(decl) => {
                    self.visit_declaration(decl)?;
                }
            }
        }
        self.level -= 1;
        writeln!(self.writer, "{}}}", " ".repeat(self.level * self.indent),)?;
        Ok(())
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule<'a>) -> std::io::Result<()> {
        writeln!(
            self.writer,
            "{}{} {} {{",
            " ".repeat(self.level * self.indent),
            at_rule.name,
            at_rule.params,
        )?;
        self.level += 1;
        for child in at_rule.children.iter_mut() {
            match child {
                RuleOrAtRuleOrDecl::Rule(rule) => {
                    self.visit_rule(rule)?;
                }
                RuleOrAtRuleOrDecl::AtRule(at_rule) => {
                    self.visit_at_rule(at_rule)?;
                }
                RuleOrAtRuleOrDecl::Declaration(_decl) => {
                    //   self.visit_declaration(decl);
                }
            }
        }
        self.level -= 1;
        writeln!(self.writer, "{}}}", " ".repeat(self.level * self.indent),)
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) -> std::io::Result<()> {
        writeln!(
            self.writer,
            "{}{}: {};",
            " ".repeat(self.level * self.indent),
            decl.prop,
            decl.value
        )
    }
}
