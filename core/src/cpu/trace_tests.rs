use std::{rc::Rc, cell::RefCell};

use crate::{cpu::{CPU, trace::Trace}, memory::Memory, bus::cpu_bus::CPUBus};

#[test]
   fn test_trace() {
       let bus = Rc::new(RefCell::new(CPUBus::new()));
       bus.borrow_mut().mem_write(100, 0xa2);
       bus.borrow_mut().mem_write(101, 0x01);
       bus.borrow_mut().mem_write(102, 0xca);
       bus.borrow_mut().mem_write(103, 0x88);
       bus.borrow_mut().mem_write(104, 0x00);

       let mut cpu = CPU::new();
       cpu.connect_bus(&bus);
       cpu.program_counter = 0x64;
       cpu.register_a = 1;
       cpu.register_x = 2;
       cpu.register_y = 3;
       let mut result: Vec<String> = vec![];
       cpu.run_with_callback(|cpu| {
           result.push(cpu.trace());
       }).unwrap();
       assert_eq!(
           "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
           result[0]
       );
       assert_eq!(
           "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
           result[1]
       );
       assert_eq!(
           "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
           result[2]
       );
   }