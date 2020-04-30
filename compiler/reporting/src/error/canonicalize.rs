use roc_collections::all::MutSet;
use roc_problem::can::PrecedenceProblem::BothNonAssociative;
use roc_problem::can::{Problem, RuntimeError};
use std::path::PathBuf;

use crate::report::{Annotation, Report, RocDocAllocator, RocDocBuilder};
use ven_pretty::DocAllocator;

pub fn can_problem<'b>(
    alloc: &'b RocDocAllocator<'b>,
    filename: PathBuf,
    problem: Problem,
) -> Report<'b> {
    let doc = match problem {
        Problem::UnusedDef(symbol, region) => {
            let line =
                r#" then remove it so future readers of your code don't wonder why it is there."#;

            alloc.stack(vec![
                alloc
                    .symbol_unqualified(symbol)
                    .append(alloc.reflow(" is not used anywhere in your code.")),
                alloc.region(region),
                alloc
                    .reflow("If you didn't intend on using ")
                    .append(alloc.symbol_unqualified(symbol))
                    .append(alloc.reflow(line)),
            ])
        }
        Problem::UnusedImport(module_id, region) => alloc.concat(vec![
            alloc.reflow("Nothing from "),
            alloc.module(module_id),
            alloc.reflow(" is used in this module."),
            alloc.region(region),
            alloc.reflow("Since "),
            alloc.module(module_id),
            alloc.reflow(" isn't used, you don't need to import it."),
        ]),
        Problem::UnusedArgument(closure_symbol, argument_symbol, region) => {
            let line = "\". Adding an underscore at the start of a variable name is a way of saying that the variable is not used.";

            alloc.concat(vec![
                alloc.symbol_unqualified(closure_symbol),
                alloc.reflow(" doesn't use "),
                alloc.symbol_unqualified(argument_symbol),
                alloc.reflow("."),
                alloc.region(region),
                alloc.reflow("If you don't need "),
                alloc.symbol_unqualified(argument_symbol),
                alloc.reflow(", then you can just remove it. However, if you really do need "),
                alloc.symbol_unqualified(argument_symbol),
                alloc.reflow(" as an argument of "),
                alloc.symbol_unqualified(closure_symbol),
                alloc.reflow(", prefix it with an underscore, like this: \"_"),
                alloc.symbol_unqualified(argument_symbol),
                alloc.reflow(line),
            ])
        }
        Problem::PrecedenceProblem(BothNonAssociative(region, left_bin_op, right_bin_op)) => alloc
            .stack(vec![
                if left_bin_op.value == right_bin_op.value {
                    alloc.concat(vec![
                        alloc.reflow("Using more than one "),
                        alloc.binop(left_bin_op.value),
                        alloc.reflow(concat!(
                            " like this requires parentheses,",
                            " to clarify how things should be grouped.",
                        )),
                    ])
                } else {
                    alloc.concat(vec![
                        alloc.reflow("Using "),
                        alloc.binop(left_bin_op.value),
                        alloc.reflow(" and "),
                        alloc.binop(right_bin_op.value),
                        alloc.reflow(concat!(
                            " together requires parentheses, ",
                            "to clarify how they should be grouped."
                        )),
                    ])
                },
                alloc.region(region),
            ]),
        Problem::UnsupportedPattern(pattern_type, region) => {
            use roc_parse::pattern::PatternType::*;

            let this_thing = match pattern_type {
                TopLevelDef => "a top-level definition:",
                DefExpr => "a value definition:",
                FunctionArg => "function arguments:",
                WhenBranch => unreachable!("all patterns are allowed in a When"),
            };

            let suggestion = vec![
                alloc.reflow(
                    "Patterns like this don't cover all possible shapes of the input type. Use a ",
                ),
                alloc.keyword("when"),
                alloc.reflow(" ... "),
                alloc.keyword("is"),
                alloc.reflow(" instead."),
            ];

            alloc.stack(vec![
                alloc
                    .reflow("This pattern is not allowed in ")
                    .append(alloc.reflow(this_thing)),
                alloc.region(region),
                alloc.concat(suggestion),
            ])
        }
        Problem::ShadowingInAnnotation {
            original_region,
            shadow,
        } => pretty_runtime_error(
            alloc,
            RuntimeError::Shadowing {
                original_region,
                shadow,
            },
        ),
        Problem::CyclicAlias(symbol, region, others) => {
            let (doc, title) = crate::error::r#type::cyclic_alias(alloc, symbol, region, others);

            return Report {
                filename,
                title,
                doc,
            };
        }
        Problem::PhantomTypeArgument {
            alias,
            variable_region,
            variable_name,
        } => alloc.stack(vec![
            alloc.concat(vec![
                alloc.reflow("The "),
                alloc.type_variable(variable_name),
                alloc.reflow(" type variable is not used in the "),
                alloc.symbol_unqualified(alias),
                alloc.reflow(" alias definition:"),
            ]),
            alloc.region(variable_region),
            alloc.reflow("Roc does not allow unused type parameters!"),
            // TODO add link to this guide section
            alloc.hint().append(alloc.reflow(
                "If you want an unused type parameter (a so-called \"phantom type\"), \
                read the guide section on phantom data.",
            )),
        ]),
        Problem::DuplicateRecordFieldValue {
            field_name,
            field_region,
            record_region,
            replaced_region,
        } => alloc.stack(vec![
            alloc.concat(vec![
                alloc.reflow("This record defines the "),
                alloc.record_field(field_name.clone()),
                alloc.reflow(" field twice!"),
            ]),
            alloc.region_all_the_things(
                record_region,
                replaced_region,
                field_region,
                Annotation::Error,
            ),
            alloc.reflow("In the rest of the program, I will only use the latter definition:"),
            alloc.region_all_the_things(
                record_region,
                field_region,
                field_region,
                Annotation::TypoSuggestion,
            ),
            alloc.concat(vec![
                alloc.reflow("For clarity, remove the previous "),
                alloc.record_field(field_name),
                alloc.reflow(" definitions from this record."),
            ]),
        ]),
        Problem::DuplicateRecordFieldType {
            field_name,
            field_region,
            record_region,
            replaced_region,
        } => alloc.stack(vec![
            alloc.concat(vec![
                alloc.reflow("This record type defines the "),
                alloc.record_field(field_name.clone()),
                alloc.reflow(" field twice!"),
            ]),
            alloc.region_all_the_things(
                record_region,
                replaced_region,
                field_region,
                Annotation::Error,
            ),
            alloc.reflow("In the rest of the program, I will only use the latter definition:"),
            alloc.region_all_the_things(
                record_region,
                field_region,
                field_region,
                Annotation::TypoSuggestion,
            ),
            alloc.concat(vec![
                alloc.reflow("For clarity, remove the previous "),
                alloc.record_field(field_name),
                alloc.reflow(" definitions from this record type."),
            ]),
        ]),
        Problem::DuplicateTag {
            tag_name,
            tag_union_region,
            tag_region,
            replaced_region,
        } => alloc.stack(vec![
            alloc.concat(vec![
                alloc.reflow("This tag union type defines the "),
                alloc.tag_name(tag_name.clone()),
                alloc.reflow(" tag twice!"),
            ]),
            alloc.region_all_the_things(
                tag_union_region,
                replaced_region,
                tag_region,
                Annotation::Error,
            ),
            alloc.reflow("In the rest of the program, I will only use the latter definition:"),
            alloc.region_all_the_things(
                tag_union_region,
                tag_region,
                tag_region,
                Annotation::TypoSuggestion,
            ),
            alloc.concat(vec![
                alloc.reflow("For clarity, remove the previous "),
                alloc.tag_name(tag_name),
                alloc.reflow(" definitions from this tag union type."),
            ]),
        ]),
        Problem::RuntimeError(runtime_error) => pretty_runtime_error(alloc, runtime_error),
    };

    Report {
        title: "SYNTAX PROBLEM".to_string(),
        filename,
        doc,
    }
}

fn pretty_runtime_error<'b>(
    alloc: &'b RocDocAllocator<'b>,
    runtime_error: RuntimeError,
) -> RocDocBuilder<'b> {
    match runtime_error {
        RuntimeError::Shadowing {
            original_region,
            shadow,
        } => {
            let line = r#"Since these variables have the same name, it's easy to use the wrong one on accident. Give one of them a new name."#;

            alloc.stack(vec![
                alloc
                    .text("The ")
                    .append(alloc.ident(shadow.value))
                    .append(alloc.reflow(" name is first defined here:")),
                alloc.region(original_region),
                alloc.reflow("But then it's defined a second time here:"),
                alloc.region(shadow.region),
                alloc.reflow(line),
            ])
        }

        RuntimeError::LookupNotInScope(loc_name, options) => {
            not_found(alloc, loc_name.region, &loc_name.value, "value", options)
        }
        RuntimeError::CircularDef(mut idents, regions) => {
            let first = idents.remove(0);

            if idents.is_empty() {
                alloc
                    .reflow("The ")
                    .append(alloc.ident(first.value.clone()))
                    .append(alloc.reflow(
                        " value is defined directly in terms of itself, causing an infinite loop.",
                    ))
            // TODO "are you trying to mutate a variable?
            // TODO hint?
            } else {
                alloc.stack(vec![
                    alloc
                        .reflow("The ")
                        .append(alloc.ident(first.value.clone()))
                        .append(
                            alloc.reflow(" definition is causing a very tricky infinite loop:"),
                        ),
                    alloc.region(regions[0].0),
                    alloc
                        .reflow("The ")
                        .append(alloc.ident(first.value.clone()))
                        .append(alloc.reflow(
                            " value depends on itself through the following chain of definitions:",
                        )),
                    crate::report::cycle(
                        alloc,
                        4,
                        alloc.ident(first.value),
                        idents
                            .into_iter()
                            .map(|ident| alloc.ident(ident.value))
                            .collect::<Vec<_>>(),
                    ),
                    // TODO hint?
                ])
            }
        }
        other => {
            //    // Example: (5 = 1 + 2) is an unsupported pattern in an assignment; Int patterns aren't allowed in assignments!
            //    UnsupportedPattern(Region),
            //    UnrecognizedFunctionName(Located<InlinableString>),
            //    SymbolNotExposed {
            //        module_name: InlinableString,
            //        ident: InlinableString,
            //        region: Region,
            //    },
            //    ModuleNotImported {
            //        module_name: InlinableString,
            //        ident: InlinableString,
            //        region: Region,
            //    },
            //    InvalidPrecedence(PrecedenceProblem, Region),
            //    MalformedIdentifier(Box<str>, Region),
            //    MalformedClosure(Region),
            //    FloatOutsideRange(Box<str>),
            //    IntOutsideRange(Box<str>),
            //    InvalidHex(std::num::ParseIntError, Box<str>),
            //    InvalidOctal(std::num::ParseIntError, Box<str>),
            //    InvalidBinary(std::num::ParseIntError, Box<str>),
            //    QualifiedPatternIdent(InlinableString),
            //    CircularDef(
            //        Vec<Located<Ident>>,
            //        Vec<(Region /* pattern */, Region /* expr */)>,
            //    ),
            //
            //    /// When the author specifies a type annotation but no implementation
            //    NoImplementation,
            todo!("TODO implement run time error reporting for {:?}", other)
        }
    }
}

fn not_found<'b>(
    alloc: &'b RocDocAllocator<'b>,
    region: roc_region::all::Region,
    name: &str,
    thing: &'b str,
    options: MutSet<Box<str>>,
) -> RocDocBuilder<'b> {
    use crate::error::r#type::suggest;

    let mut suggestions = suggest::sort(name, options.iter().map(|v| v.as_ref()).collect());
    suggestions.truncate(4);

    let default_no = alloc.concat(vec![
        alloc.reflow("Is there an "),
        alloc.keyword("import"),
        alloc.reflow(" or "),
        alloc.keyword("exposing"),
        alloc.reflow(" missing up-top"),
    ]);

    let default_yes = alloc.reflow("these names seem close though:");

    let to_details = |no_suggestion_details, yes_suggestion_details| {
        if suggestions.is_empty() {
            no_suggestion_details
        } else {
            alloc.stack(vec![
                yes_suggestion_details,
                alloc
                    .vcat(suggestions.into_iter().map(|v| alloc.string(v.to_string())))
                    .indent(4),
            ])
        }
    };

    alloc.stack(vec![
        alloc.concat(vec![
            alloc.reflow("I cannot find a `"),
            alloc.string(name.to_string()),
            alloc.reflow("` "),
            alloc.reflow(thing),
        ]),
        alloc.region(region),
        to_details(default_no, default_yes),
    ])
}