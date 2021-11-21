use std::{borrow::Cow, io::Write};

use recursive_parser::{parser::*, visitor::VisitMut};
use regex::{Captures, Regex};

#[derive(Default)]
pub struct AspectRatioMini {}

impl<'a> VisitMut<'a> for AspectRatioMini {
    fn visit_root(&mut self, root: &mut Root<'a>) {
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
    }

    fn visit_rule(&mut self, rule: &mut Rule<'a>) {
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
                    self.visit_declaration(decl);
                }
            });
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule<'a>) {
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
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
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
            return;
        }
        let value = process_ration_value(&decl.value.content);
        process_ratio_conf(decl, value);
    }
}

// // 解析 aspect-ratio 的值，支持 : | / 三种分隔符，分隔符前后可以有一个或多个空格，例如：
// // 16:9, 16 | 9, 16 / 9
// function processRatioValue(css, rule, decl) {
//   var ratio = null
//   var re = /['"]?(((?:\d*\.?\d*)?)(?:\s*[\:\|\/]\s*)(\d*\.?\d*))['"]?/g

//   ratio = decl.value
//   ratio = ratio.replace(re, function(match, r, x, y) {
//     return NP.times(NP.divide(y, x), 100) + '%' // Use number-precision module to fix JS calculation precision problem.
//   })

//   return ratio
// }

fn process_ration_value<'a>(decl: &'a Cow<'a, str>) -> String {
    let re = Regex::new(r#"['"]?(?:((?:\d*\.?\d*)?)(?:\s*[:|/]\s*)(\d*\.?\d*))['"]?"#).unwrap();
    re.replace_all(&decl, |caps: &Captures| {
        let computed_result = caps[2]
            .parse::<f32>()
            .and_then(|y| caps[1].parse::<f32>().and_then(|x| Ok(y / x * 100f32)));
        match computed_result {
            Ok(value) => value.to_string() + "%",
            Err(_) => decl.to_string(),
        }
    })
    .to_string()
}

// function processRatioConf(css, rule, decl, ratio) {
//   var sels = []

//   ratio.pseudo = clone(defaults.pseudo)
//   ratio.pseudo.source = decl.source

//   for (var i = 0; i < rule.selectors.length; i++) {
//     sels.push(rule.selectors[i] + ':before')
//   }

//   ratio.pseudo.selector = sels.join(', ')
// }

fn process_ratio_conf<'a>(decl: &mut Declaration<'a>, ratio: String) {
    decl.prop.content = Cow::Owned(format!("padding-top"));
    decl.value.content = Cow::Owned(ratio);
}

#[derive(Default)]
pub struct SimplePrettier<W: Write> {
    level: usize,
    pub writer: W,
}

impl<W: Write> SimplePrettier<W> {
    pub fn new(writer: W) -> Self {
        Self { level: 0, writer }
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
        self.writer.write(
            format!(
                "{}{} {}\n",
                " ".repeat(self.level * 2),
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
        write!(self.writer, "{}{}\n", " ".repeat(self.level * 2), "}")?;
        Ok(())
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule<'a>) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{} {} {}\n",
            " ".repeat(self.level * 2),
            at_rule.name,
            at_rule.params,
            "{"
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
        write!(self.writer, "{}{}\n", " ".repeat(self.level * 2), "}")
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) -> std::io::Result<()> {
        write!(
            self.writer,
            "{}{} : {};\n",
            " ".repeat(self.level * 2),
            decl.prop,
            decl.value
        )
    }
}
