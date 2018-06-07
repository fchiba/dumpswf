extern crate swf;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use swf::avm1::types::*;
use swf::Tag::*;
use swf::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
    let f = File::open(path).unwrap();
    let reader = BufReader::new(f);
    let swf = swf::read_swf(reader).unwrap();

    println!("root_mc");
    print_tags(&swf.tags, swf.version);
    for tag in swf.tags {
        match tag {
            DefineSprite(sprite) => {
                println!("sprite id={}", sprite.id);
                print_tags(&sprite.tags, swf.version);
            }
            DefineButton(button) | DefineButton2(button) => {
                println!("button id={}", button.id);
                for button_action in button.actions {
                    println!("  button cond={:?}", button_action.conditions);
                    print_action_data(&button_action.action_data, swf.version);
                }
            }
            _ => {}
        }
    }
}

fn print_tags(tags: &Vec<Tag>, version: u8) {
    let mut frame = 1;
    for tag in tags {
        match tag {
            ShowFrame => {
                frame += 1;
            }
            DoAction(data) => {
                println!("  frame {}", frame);
                print_action_data(data, version);
            }
            _ => {}
        }
    }
}

fn print_action_data(action_data: &Vec<u8>, version: u8) {
    let mut reader = swf::avm1::read::Reader::new(&action_data[..], version);
    let actions = reader.read_action_list().unwrap();
    print_action(&actions, Vec::new(), 4);
}

fn print_action(actions: &Vec<ActionWithSize>, constant_pool: Vec<String>, level: usize) {
    let indent: String = std::iter::repeat(" ").take(level).collect();

    let positions: Vec<i64> = {
        let mut pos = 0;
        actions
            .iter()
            .map(|action| {
                pos += action.size;
                pos as i64
            })
            .collect()
    };
    let position_to_idx: HashMap<i64, usize> = positions
        .iter()
        .enumerate()
        .map(|(idx, pos)| (*pos, idx))
        .collect();

    use swf::avm1::types::Value;

    let mut constant_pool = constant_pool;
    for (idx, action) in actions.iter().enumerate() {
        use swf::avm1::types::Action::*;
        match &action.action {
            ConstantPool(pool) => {
                constant_pool = pool.clone();
                println!("{}{}: ConstantPool", indent, idx);
            }
            Push(values) => {
                let values: Vec<_> = values
                    .iter()
                    .map(|v| match v {
                        Value::ConstantPool(idx) => {
                            Value::Str(constant_pool[*idx as usize].clone())
                        }
                        _ => v.clone(),
                    })
                    .collect();
                println!("{}{}: Push {:?}", indent, idx, values);
            }
            If { offset } => {
                let current_pos = positions[idx];
                let next_pos = current_pos + *offset as i64;
                let next_idx = position_to_idx[&next_pos] + 1;
                println!("{}{}: If (to:{})", indent, idx, next_idx);
            }
            Jump { offset } => {
                let current_pos = positions[idx];
                let next_pos = current_pos + *offset as i64;
                let next_idx = position_to_idx[&next_pos] + 1;
                println!("{}{}: Jump (to:{})", indent, idx, next_idx);
            }
            DefineFunction {
                name,
                params,
                actions,
            } => {
                println!(
                    "{}{}: DefineFunction name={} params={:?}",
                    indent, idx, name, params
                );
                print_action(actions, constant_pool.clone(), level + 4);
            }
            DefineFunction2(function) => {
                println!("{}{}: DefineFunction2", indent, idx);
                println!("{}      name={}", indent, function.name);
                let params: Vec<_> = function
                    .params
                    .iter()
                    .map(|param| {
                        format!(
                            "{}({})",
                            param.name,
                            param
                                .register_index
                                .map(|i| i.to_string())
                                .unwrap_or("-".to_string())
                        )
                    })
                    .collect();
                println!("{}      params={:?}", indent, params);
                let mut preloads = Vec::new();
                if function.preload_parent {
                    preloads.push("parent");
                }
                if function.preload_root {
                    preloads.push("root");
                }
                if function.preload_super {
                    preloads.push("super");
                }
                if function.preload_arguments {
                    preloads.push("arguments");
                }
                if function.preload_this {
                    preloads.push("this");
                }
                if function.preload_global {
                    preloads.push("global");
                }
                println!("{}      preloads={:?}", indent, preloads);
                print_action(&function.actions, constant_pool.clone(), level + 4);
            }
            _ => {
                println!("{}{}: {:?}", indent, idx, action.action);
            }
        }
    }
}
