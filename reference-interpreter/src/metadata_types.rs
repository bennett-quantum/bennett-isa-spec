//! Type admission for §9.1 metadata, before identities or semantic consumers.
//! Cross-field, name-resolution and binding checks remain in their validators.
use crate::{err, json::Json, metadata::nonnegative, Object, Result};
type Check = fn(&Json) -> Result<()>;
fn text(v: &Json) -> Result<()> {
    v.string().map(|_| ())
}
fn uint(v: &Json) -> Result<()> {
    v.uint_text().map(|_| ())
}
fn decimal(v: &Json) -> Result<()> {
    nonnegative(v)
}
fn signed_decimal(v: &Json) -> Result<()> {
    match v {
        Json::Number(s) | Json::String(s) => {
            nonnegative(&Json::String(s.strip_prefix('-').unwrap_or(s).into()))
        }
        _ => err("§9.1", "expected decimal endpoint"),
    }
}
fn record(v: &Json, required: &[(&str, Check)], optional: &[(&str, Check)]) -> Result<()> {
    v.fields(
        &required.iter().map(|p| p.0).collect::<Vec<_>>(),
        &optional.iter().map(|p| p.0).collect::<Vec<_>>(),
    )?;
    for (key, check) in required {
        check(v.get(key)?)?;
    }
    for (key, check) in optional {
        if let Some(v) = v.object()?.get(*key) {
            check(v)?;
        }
    }
    Ok(())
}
fn array(v: &Json, check: Check) -> Result<()> {
    for x in v.array()? {
        check(x)?;
    }
    Ok(())
}
fn map(v: &Json, check: Check) -> Result<()> {
    for x in v.object()?.values() {
        check(x)?;
    }
    Ok(())
}
fn texts(v: &Json) -> Result<()> {
    array(v, text)
}
fn uints(v: &Json) -> Result<()> {
    array(v, uint)
}
fn object(v: &Json) -> Result<()> {
    v.object().map(|_| ())
}
fn list(v: &Json) -> Result<()> {
    v.array().map(|_| ())
}
fn selector(v: &Json) -> Result<()> {
    if v == &Json::String("*".into()) {
        Ok(())
    } else {
        uint(v)
    }
}
fn iteration(v: &Json) -> Result<()> {
    if v == &Json::String("*".into()) {
        Ok(())
    } else {
        uints(v)
    }
}
fn point(v: &Json) -> Result<()> {
    let a = v.array()?;
    if a.len() != 3 {
        return err("§9.1", "program point needs three components");
    }
    text(&a[0])?;
    text(&a[1])?;
    uint(&a[2])
}
fn points(v: &Json) -> Result<()> {
    array(v, point)
}
fn path(v: &Json) -> Result<()> {
    array(v, |x| {
        record(
            x,
            &[("symbol", text), ("point", text), ("iteration", iteration)],
            &[],
        )
    })
}
fn mem(v: &Json) -> Result<()> {
    array(v, |x| {
        record(x, &[("A", uint), ("W", uint), ("role", text)], &[])
    })
}
fn signature(v: &Json) -> Result<()> {
    let mut fields: Vec<(&str, Check)> = vec![("abi", text), ("tier", text), ("kind", text)];
    let extra: &[(&str, Check)] = match v.get("kind")?.string()? {
        "data" => &[("size", uint)],
        "oracle" => &[
            ("in", uints),
            ("tgt", uint),
            ("fault", uint),
            ("mem", mem),
            ("args", texts),
        ],
        "action" | "perm_action" => &[
            ("mut", uints),
            ("in", uints),
            ("mem", mem),
            ("args", texts),
            ("bounds", uints),
        ],
        "lproc" => &[("params", uints), ("result", uint)],
        "cc" | "r" => &[("params", texts), ("results", texts)],
        _ => return err("§9.1", "unknown signature kind"),
    };
    fields.extend_from_slice(extra);
    record(v, &fields, &[])
}
fn symbols(v: &Json) -> Result<()> {
    map(v, |s| {
        record(
            s,
            &[
                ("name", text),
                ("kind", text),
                ("signature", signature),
                ("interface_hash", text),
            ],
            &[],
        )
    })
}
fn imports(v: &Json) -> Result<()> {
    map(v, |s| {
        record(
            s,
            &[("signature", signature)],
            &[("pinned_closure_hash", text)],
        )
    })
}
fn devices(v: &Json) -> Result<()> {
    fn versions(v: &Json) -> Result<()> {
        map(v, |x| record(x, &[("version", text)], &[]))
    }
    record(v, &[("models", versions), ("services", versions)], &[])
}
fn layout(v: &Json) -> Result<()> {
    fn globals(v: &Json) -> Result<()> {
        map(v, |x| {
            record(x, &[("base", uint), ("size", uint), ("image", text)], &[])
        })
    }
    fn regions(v: &Json) -> Result<()> {
        map(v, |x| map(x, |r| record(r, &[("base", uint)], &[])))
    }
    record(
        v,
        &[
            ("ptr_width", uint),
            ("H", uint),
            ("A_tab", uint),
            ("S_cells", uint),
            ("null_size", uint),
            ("heap_base", uint),
            ("heap_limit_max", uint),
            ("globals", globals),
            ("interface_regions", regions),
        ],
        &[("block_image", text)],
    )
}
fn named_version(v: &Json) -> Result<()> {
    record(v, &[("name", text), ("version", text)], &[])
}
fn expansion(v: &Json) -> Result<()> {
    record(v, &[("name", text), ("version", text), ("hash", text)], &[])
}
fn schedule(v: &Json) -> Result<()> {
    fn bindings(v: &Json) -> Result<()> {
        record(
            v,
            &[
                ("closure_hash", text),
                ("target_hash", text),
                ("region", text),
                ("variant", text),
                ("inherited_controls", uint),
                ("budget", uints),
                ("params_hash", text),
                ("epsilon", decimal),
                ("scheduler", named_version),
                ("expansion", expansion),
            ],
            &[],
        )
    }
    fn entries(v: &Json) -> Result<()> {
        array(v, |e| {
            record(
                e,
                &[
                    ("path", path),
                    ("step", selector),
                    ("phase", text),
                    ("generated", selector),
                    ("kind", text),
                ],
                &[
                    ("cleanup", |x| record(x, &[("realisation", text)], &[])),
                    ("gate", |x| {
                        record(x, &[("realisation", text), ("epsilon_share", decimal)], &[])
                    }),
                    ("alloc", |x| record(x, &[("class", text)], &[])),
                    ("start", |x| record(x, &[("time", decimal)], &[])),
                ],
            )
        })
    }
    record(v, &[("bindings", bindings), ("entries", entries)], &[])
}
fn domain(v: &Json) -> Result<()> {
    fn bundles(v: &Json) -> Result<()> {
        array(v, |b| {
            record(
                b,
                &[("name", text), ("width", uint), ("lo", uint), ("hi", uint)],
                &[],
            )
        })
    }
    fn cells(v: &Json) -> Result<()> {
        if v == &Json::String("all".into()) {
            Ok(())
        } else {
            array(v, |c| {
                record(c, &[("index", uint), ("lo", uint), ("hi", uint)], &[])
            })
        }
    }
    fn regions(v: &Json) -> Result<()> {
        array(v, |r| record(r, &[("name", text), ("cells", cells)], &[]))
    }
    record(
        v,
        &[
            ("bundles", bundles),
            ("borrowed", bundles),
            ("regions", regions),
        ],
        &[],
    )
}
fn subject(v: &Json) -> Result<()> {
    record(
        v,
        &[
            ("closure_hash", text),
            ("variant", text),
            ("inherited_controls", uint),
            ("schedule_hash", text),
        ],
        &[],
    )
}
fn claim(v: &Json) -> Result<()> {
    let label = &[("label", text as Check)];
    match v.get("kind")?.string()? {
        "exhaustive" => {
            fn expected(v: &Json) -> Result<()> {
                if v == &Json::String("oracle".into()) {
                    return Ok(());
                }
                if v.object()?.contains_key("table") {
                    record(v, &[("table", list)], &[])
                } else {
                    record(v, &[("program", text)], &[])
                }
            }
            record(
                v,
                &[("kind", text), ("domain", domain), ("expected", expected)],
                &[("label", text), ("subject", subject)],
            )
        }
        "sampled" => record(
            v,
            &[
                ("kind", text),
                ("domain", domain),
                ("sampler", |x| {
                    record(
                        x,
                        &[("algorithm", text), ("seed", texts), ("count", uint)],
                        &[],
                    )
                }),
            ],
            label,
        ),
        "structural" => {
            record(
                v,
                &[
                    ("kind", text),
                    ("rule", text),
                    ("subclaims", uints),
                    ("witness", object),
                ],
                label,
            )?;
            let w = v.get("witness")?;
            match v.get("rule")?.string()? {
                "adjoint_table" => record(w, &[], &[]),
                "inverse_pair_library" => record(
                    w,
                    &[("library_closure_hash", text), ("claim_index", uint)],
                    &[],
                ),
                "translation_lemma" => record(
                    w,
                    &[
                        ("V", points),
                        ("W", points),
                        ("X", texts),
                        ("B", texts),
                        ("frame", points),
                    ],
                    &[],
                ),
                _ => err("§9.1", "unknown structural rule"),
            }
        }
        "numerical" | "channel" => {
            let mut fields: Vec<(&str, Check)> = vec![
                ("kind", text),
                ("subject", subject),
                ("metric", text),
                ("precision_bits", uint),
                ("bound", decimal),
            ];
            if v.get("kind")?.string()? == "channel" {
                fields.push(("dual", |v| {
                    array(v, |x| {
                        record(
                            x,
                            &[
                                ("Y", |v| array(v, |v| array(v, signed_decimal))),
                                ("mu", decimal),
                            ],
                            &[],
                        )
                    })
                }));
            }
            record(v, &fields, label)
        }
        _ => err("§9.1", "unknown claim kind"),
    }
}
fn expression(v: &Json) -> Result<()> {
    crate::certificate::expression(v, &std::collections::BTreeSet::from(["c"]))
}
fn frame(v: &Json) -> Result<()> {
    fn counts(v: &Json) -> Result<()> {
        map(v, expression)
    }
    fn scratch(v: &Json) -> Result<()> {
        map(v, |x| {
            if x.object()
                .is_ok_and(|m| m.contains_key("clean") || m.contains_key("dirty"))
            {
                record(x, &[("clean", expression), ("dirty", expression)], &[])
            } else {
                expression(x)
            }
        })
    }
    record(
        v,
        &[
            ("interface", |v| {
                array(v, |s| {
                    record(
                        s,
                        &[("role", text), ("width", uint), ("class", text)],
                        &[("name", text)],
                    )
                })
            }),
            ("scratch", scratch),
            ("retained_history", expression),
            ("borrow", counts),
            ("scheme_ancillae", counts),
        ],
        &[("live_ranges", |v| {
            array(v, |r| {
                record(r, &[("frame", text), ("start", uint), ("end", uint)], &[])
            })
        })],
    )
}
fn metrics(v: &Json) -> Result<()> {
    record(v, &[], &[("operator_norm", decimal), ("diamond", decimal)])
}
fn region(v: &Json) -> Result<()> {
    fn verification(v: &Json) -> Result<()> {
        record(
            v,
            &[
                (
                    "oracle",
                    |x| if x == &Json::Null { Ok(()) } else { uint(x) },
                ),
                ("borrow_frames", uints),
                ("inverse_pairs", |v| {
                    array(v, |p| {
                        record(p, &[("g", text), ("ginv", text), ("claim", uint)], &[])
                    })
                }),
                ("unitary", uints),
                ("channel", uints),
                ("fault_rate", |v| {
                    record(
                        v,
                        &[
                            ("domain", domain),
                            ("faults_by_code", |v| map(v, uint)),
                            ("undefined_excluded", uint),
                            ("layout_dependent_excluded", uint),
                        ],
                        &[],
                    )
                }),
            ],
            &[],
        )
    }
    record(
        v,
        &[
            ("kind", text),
            ("signature", signature),
            ("bounds", uints),
            ("frame", frame),
            ("resources", |v| map(v, decimal)),
            ("gates_used", |v| {
                map(v, |g| {
                    record(
                        g,
                        &[("count", uint)],
                        &[("scheme", text), ("epsilon", decimal)],
                    )
                })
            }),
            ("claims", |v| array(v, claim)),
            ("verification", verification),
            ("cleanup", |v| {
                map(v, |v| {
                    array(v, |c| {
                        record(c, &[("path", path), ("realisation", text)], &[])
                    })
                })
            }),
            ("error", |v| {
                map(v, |v| {
                    record(v, &[("local", metrics), ("inclusive", metrics)], &[])
                })
            }),
            ("definedness_assumptions", texts),
        ],
        &[],
    )
}
fn certificate(v: &Json) -> Result<()> {
    fn bindings(v: &Json) -> Result<()> {
        record(
            v,
            &[
                ("object_hash", text),
                ("interface_hash", text),
                ("closure_hash", text),
                ("reference_program_hash", text),
                ("dialect_mapping_version", text),
                ("target_hash", text),
            ],
            &[("schedule_hash", text)],
        )
    }
    record(
        v,
        &[
            ("versions", |v| {
                record(
                    v,
                    &[
                        ("isa", text),
                        ("object", text),
                        ("launch", text),
                        ("sched", text),
                        ("cert", text),
                        ("coverage", text),
                        ("target", text),
                    ],
                    &[],
                )
            }),
            ("bindings", bindings),
            ("producer", |v| {
                record(
                    v,
                    &[
                        ("name", text),
                        ("version", text),
                        ("materialisation", |v| {
                            record(
                                v,
                                &[
                                    ("llvm_version", text),
                                    ("pipeline", text),
                                    ("normaliser_version", text),
                                ],
                                &[],
                            )
                        }),
                    ],
                    &[],
                )
            }),
            ("regions", |v| map(v, region)),
            ("composition", |v| {
                record(
                    v,
                    &[
                        ("multiplicities", |v| map(v, uint)),
                        ("epsilon_total", |v| record(v, &[("diamond", decimal)], &[])),
                        ("method", text),
                    ],
                    &[],
                )
            }),
        ],
        &[],
    )
}
fn coverage(v: &Json) -> Result<()> {
    record(
        v,
        &[("entries", |v| {
            array(v, |e| {
                record(
                    e,
                    &[
                        ("point", point),
                        ("source", text),
                        ("classification", text),
                        ("reason", text),
                        ("count", uint),
                    ],
                    &[("fault_code", uint), ("shim_hash", text)],
                )
            })
        })],
        &[],
    )
}
impl Object {
    pub(crate) fn validate_metadata_types(&self) -> Result<()> {
        for (section, value) in &self.metadata {
            let check: Check = match section.as_str() {
                "symbols" => symbols,
                "imports" => imports,
                "devices" => devices,
                "layout" => layout,
                "schedule" => schedule,
                "certificate" => certificate,
                "coverage" => coverage,
                _ => return err("§9.1", "unknown metadata section"),
            };
            let rule = match section.as_str() {
                "schedule" => "§11",
                "certificate" => "§12/§9.4",
                "coverage" => "§13",
                _ => "§9.1",
            };
            check(value)
                .map_err(|e| crate::Error(format!("§9.1: {section} schema ({rule}): {e}")))?;
        }
        Ok(())
    }
}
