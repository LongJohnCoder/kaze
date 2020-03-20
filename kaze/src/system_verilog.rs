//! SystemVerilog code generation.

mod compiler;
mod instance_decls;
mod ir;

use compiler::*;
use instance_decls::*;
use ir::*;

use crate::code_writer;
use crate::graph;
use crate::validation::*;

use std::collections::HashMap;
use std::io::{Result, Write};

// TODO: Note that mutable writer reference can be passed, see https://rust-lang.github.io/api-guidelines/interoperability.html#c-rw-value
pub fn generate<'a, W: Write>(m: &'a graph::Module<'a>, w: W) -> Result<()> {
    validate_module_hierarchy(m);

    let mut instances = HashMap::new();
    for instance in m.instances.borrow().iter() {
        let mut input_names = HashMap::new();
        for (name, _) in instance.instantiated_module.inputs.borrow().iter() {
            input_names.insert(name.clone(), format!("__{}_input_{}", instance.name, name));
        }

        let mut output_names = HashMap::new();
        for (name, _) in instance.instantiated_module.outputs.borrow().iter() {
            output_names.insert(name.clone(), format!("__{}_output_{}", instance.name, name));
        }

        instances.insert(
            *instance,
            InstanceDecls {
                input_names,
                output_names,
            },
        );
    }

    let mut regs = HashMap::new();
    for reg in m.registers.borrow().iter() {
        match reg.data {
            graph::SignalData::Reg { data } => {
                let value_name = format!("__reg_{}_{}", data.name, regs.len());
                let next_name = format!("{}_next", value_name);
                regs.insert(
                    *reg,
                    RegisterDecls {
                        data,
                        value_name,
                        next_name,
                    },
                );
            }
            _ => unreachable!(),
        }
    }

    let module_decls = ModuleDecls { instances, regs };

    let mut c = Compiler::new();

    let mut assignments = AssignmentContext::new();
    for (name, output) in m.outputs.borrow().iter() {
        let expr = c.compile_signal(&output, &module_decls, &mut assignments);
        assignments.push(Assignment {
            target_name: name.clone(),
            expr,
        });
    }

    let mut node_decls = Vec::new();

    for (instance, instance_decls) in module_decls.instances.iter() {
        for (name, decl_name) in instance_decls.input_names.iter() {
            node_decls.push(NodeDecl {
                name: decl_name.clone(),
                bit_width: instance.instantiated_module.inputs.borrow()[name].bit_width(),
            });

            let expr = c.compile_signal(
                instance.driven_inputs.borrow()[name],
                &module_decls,
                &mut assignments,
            );
            assignments.push(Assignment {
                target_name: decl_name.clone(),
                expr,
            });
        }

        for (name, decl_name) in instance_decls.output_names.iter() {
            node_decls.push(NodeDecl {
                name: decl_name.clone(),
                bit_width: instance.instantiated_module.outputs.borrow()[name].bit_width(),
            });
        }
    }

    for reg in module_decls.regs.values() {
        node_decls.push(NodeDecl {
            name: reg.value_name.clone(),
            bit_width: reg.data.bit_width,
        });
        node_decls.push(NodeDecl {
            name: reg.next_name.clone(),
            bit_width: reg.data.bit_width,
        });

        let expr = c.compile_signal(
            reg.data.next.borrow().unwrap(),
            &module_decls,
            &mut assignments,
        );
        assignments.push(Assignment {
            target_name: reg.next_name.clone(),
            expr,
        });
    }

    let mut w = code_writer::CodeWriter::new(w);

    w.append_line(&format!("module {}(", m.name))?;
    w.indent();

    // TODO: Make conditional based on the presents of (resetable) state elements
    w.append_line("input wire logic reset_n,")?;
    w.append_indent()?;
    w.append("input wire logic clk")?;
    if !m.inputs.borrow().is_empty() || !m.outputs.borrow().is_empty() {
        w.append(",")?;
        w.append_newline()?;
    }
    w.append_newline()?;
    let inputs = m.inputs.borrow();
    let num_inputs = inputs.len();
    for (i, (name, source)) in inputs.iter().enumerate() {
        w.append_indent()?;
        w.append("input wire logic ")?;
        if source.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", source.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if !m.outputs.borrow().is_empty() || i < num_inputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    let outputs = m.outputs.borrow();
    let num_outputs = outputs.len();
    for (i, (name, output)) in outputs.iter().enumerate() {
        w.append_indent()?;
        w.append("output wire logic ")?;
        if output.bit_width() > 1 {
            w.append(&format!("[{}:{}] ", output.bit_width() - 1, 0))?;
        }
        w.append(name)?;
        if i < num_outputs - 1 {
            w.append(",")?;
        }
        w.append_newline()?;
    }
    w.append_line(");")?;
    w.append_newline()?;

    if !node_decls.is_empty() {
        for node_decl in node_decls {
            node_decl.write(&mut w)?;
        }
        w.append_newline()?;
    }

    if !module_decls.instances.is_empty() {
        for (instance, instance_decls) in module_decls.instances.iter() {
            w.append_line(&format!(
                "{} {}(",
                instance.instantiated_module.name, instance.name
            ))?;
            w.indent();
            // TODO: Make conditional based on the presents of (resetable) state elements
            w.append_line(".reset_n(reset_n),")?;
            w.append_indent()?;
            w.append(".clk(clk)")?;
            if !instance_decls.input_names.is_empty() {
                for (name, decl_name) in instance_decls.input_names.iter() {
                    w.append(",")?;
                    w.append_newline()?;
                    w.append_indent()?;
                    w.append(&format!(".{}({})", name, decl_name))?;
                }
            }
            if !instance_decls.output_names.is_empty() {
                for (name, decl_name) in instance_decls.output_names.iter() {
                    w.append(",")?;
                    w.append_newline()?;
                    w.append_indent()?;
                    w.append(&format!(".{}({})", name, decl_name))?;
                }
            }
            w.unindent()?;
            w.append(");")?;
            w.append_newline()?;
            w.append_newline()?;
        }
    }

    if !module_decls.regs.is_empty() {
        for reg in module_decls.regs.values() {
            w.append_indent()?;
            w.append("always_ff @(posedge clk")?;
            if reg.data.initial_value.borrow().is_some() {
                w.append(", negedge reset_n")?;
            }
            w.append(") begin")?;
            w.append_newline()?;
            w.indent();
            if let Some(ref initial_value) = *reg.data.initial_value.borrow() {
                w.append_line("if (~reset_n) begin")?;
                w.indent();
                w.append_line(&format!(
                    "{} <= {}'h{:0x};",
                    reg.value_name,
                    reg.data.bit_width,
                    initial_value.numeric_value()
                ))?;
                w.unindent()?;
                w.append_line("end")?;
                w.append_line("else begin")?;
                w.indent();
            }
            w.append_line(&format!("{} <= {};", reg.value_name, reg.next_name))?;
            if reg.data.initial_value.borrow().is_some() {
                w.unindent()?;
                w.append_line("end")?;
            }
            w.unindent()?;
            w.append_line("end")?;
            w.append_newline()?;
        }
    }

    if !assignments.is_empty() {
        assignments.write(&mut w)?;
        w.append_newline()?;
    }

    w.unindent()?;
    w.append_line("endmodule")?;
    w.append_newline()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::*;

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\"."
    )]
    fn recursive_module_definition_error1() {
        let c = Context::new();

        let a = c.module("A");

        let _ = a.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because it has a recursive definition formed by an instance of itself called \"a\" in module \"B\"."
    )]
    fn recursive_module_definition_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");

        let _ = a.instance("b", "B");
        let _ = b.instance("a", "A");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains an instance of module \"B\" called \"b\" whose input \"i\" is not driven."
    )]
    fn undriven_instance_input_error() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.input("i", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.reg("r", 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a register called \"r\" which is not driven."
    )]
    fn undriven_register_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.reg("r", 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error1() {
        let c = Context::new();

        let a = c.module("A");
        let _ = a.mem("m", 1, 1);

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have any read ports."
    )]
    fn mem_without_read_ports_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let _ = b.mem("m", 1, 1);

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"A\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error1() {
        let c = Context::new();

        let a = c.module("A");
        let m = a.mem("m", 1, 1);
        let _ = m.read_port(a.low(), a.low());

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"A\" because module \"B\" contains a memory called \"m\" which doesn't have initial contents or a write port specified. At least one of the two is required."
    )]
    fn mem_without_initial_contents_or_write_port_error2() {
        let c = Context::new();

        let a = c.module("A");
        let b = c.module("B");
        let m = b.mem("m", 1, 1);
        let _ = m.read_port(b.low(), b.low());

        let _ = a.instance("b", "B");

        // Panic
        generate(a, Vec::new()).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot generate code for module \"b\" because module \"a\" contains an output called \"o\" which forms a combinational loop with itself."
    )]
    fn combinational_loop_error() {
        let c = Context::new();

        let a = c.module("a");
        a.output("o", a.input("i", 1));

        let b = c.module("b");
        let a_inst = b.instance("a_inst", "a");
        let a_inst_o = a_inst.output("o");
        a_inst.drive_input("i", a_inst_o);

        // Panic
        generate(b, Vec::new()).unwrap();
    }
}
