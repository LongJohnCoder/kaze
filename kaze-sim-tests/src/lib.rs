#[cfg(test)]
mod tests {
    mod modules {
        include!(concat!(env!("OUT_DIR"), "/modules.rs"));
    }

    use modules::*;

    #[test]
    fn input_masking() {
        let mut m = InputMasking::new();

        m.i = 0xffffffff;
        m.prop();
        assert_eq!(m.o, 0x07ffffff);
    }

    #[test]
    fn widest_input() {
        let mut m = WidestInput::new();

        m.i = 0xfadebabedeadbeefabad1deabadc0de5;
        m.prop();
        assert_eq!(m.o, 0xfadebabedeadbeefabad1deabadc0de5);
    }

    #[test]
    fn add_test_module() {
        let mut m = AddTestModule::new();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o1, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o1, true);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o1, true);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o1, false);

        m.i3 = 1;
        m.i4 = 2;
        m.prop();
        assert_eq!(m.o2, 3);

        m.i3 = 0xffffu32;
        m.i4 = 0x0002u32;
        m.prop();
        assert_eq!(m.o2, 0x0001u32);

        m.i5 = 0xfade0000u32;
        m.i6 = 0x0000babeu32;
        m.prop();
        assert_eq!(m.o3, 0xfadebabeu32);

        m.i5 = 0xffffffffu32;
        m.i6 = 0x00000002u32;
        m.prop();
        assert_eq!(m.o3, 0x00000001u32);

        m.i7 = 0xfade00000000beefu64;
        m.i8 = 0x0000babedead0000u64;
        m.prop();
        assert_eq!(m.o4, 0xfadebabedeadbeefu64);

        m.i7 = 0xffffffffffffffffu64;
        m.i8 = 0x0000000000000002u64;
        m.prop();
        assert_eq!(m.o4, 0x0000000000000001u64);

        m.i9 = 0xfade00000000beef0000000000000001u128;
        m.i10 = 0x0000babedead00000000000000000002u128;
        m.prop();
        assert_eq!(m.o5, 0xfadebabedeadbeef0000000000000003u128);

        m.i9 = 0xffffffffffffffffffffffffffffffffu128;
        m.i10 = 0x00000000000000000000000000000002u128;
        m.prop();
        assert_eq!(m.o5, 0x00000000000000000000000000000001u128);

        m.i11 = 127u32;
        m.i12 = 2u32;
        m.prop();
        assert_eq!(m.o6, 1u32);
    }

    #[test]
    fn bit_and_test_module() {
        let mut m = BitAndTestModule::new();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = false;
        assert_eq!(m.o, true); // No propagation
        m.prop();
        assert_eq!(m.o, false);
    }

    #[test]
    fn bit_or_test_module() {
        let mut m = BitOrTestModule::new();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = false;
        assert_eq!(m.o, true); // No propagation
        m.prop();
        assert_eq!(m.o, false);
    }

    #[test]
    fn bit_xor_test_module() {
        let mut m = BitXorTestModule::new();

        m.i1 = false;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = false;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, true);

        m.i1 = true;
        m.i2 = true;
        m.prop();
        assert_eq!(m.o, false);

        m.i1 = true;
        m.i2 = false;
        assert_eq!(m.o, false); // No propagation
        m.prop();
        assert_eq!(m.o, true);
    }

    #[test]
    fn not_test_module() {
        let mut m = NotTestModule::new();

        m.i = 0;
        m.prop();
        assert_eq!(m.o, 0xf);

        m.i = 0xff;
        m.prop();
        assert_eq!(m.o, 0);

        m.i = 0xa;
        m.prop();
        assert_eq!(m.o, 0x5);

        m.i = 0x5;
        m.prop();
        assert_eq!(m.o, 0xa);
    }

    #[test]
    fn reg_test_module() {
        let mut m = RegTestModule::new();

        // Check initial value
        m.reset();
        m.prop();
        assert_eq!(m.o1, 0);

        // Register value doesn't change without clock edge
        m.i1 = 0xdeadbeef;
        m.prop();
        assert_eq!(m.o1, 0);
        m.posedge_clk();
        assert_eq!(m.o1, 0); // No propagation
        m.prop();
        assert_eq!(m.o1, 0xdeadbeef);
        m.i1 = 0xfadebabe;
        m.prop();
        assert_eq!(m.o1, 0xdeadbeef);

        // Clock in initial value (second reg explicitly doesn't have one!)
        m.i2 = 0xfadebabe;
        m.prop();
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o2, 0xfadebabe);

        // Register with no default value doesn't change with reset
        m.reset();
        m.prop();
        assert_eq!(m.o2, 0xfadebabe);
    }

    #[test]
    fn simple_reg_delay() {
        let mut m = SimpleRegDelay::new();

        // Check initial value
        m.reset();
        m.prop();
        assert_eq!(m.o, 0);

        // The input propagates through 3 registers, so we won't see it output for 3 cycles
        m.i = 0xffffffffffffffffffffffffffffffff;
        m.prop(); // Propagate to first register input
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0xfffffffffffffffffffffffff);
    }

    #[test]
    fn bit_test_module_0() {
        let mut m = BitTestModule0::new();

        m.i = false;
        m.prop();
        assert_eq!(m.o, false);

        m.i = true;
        m.prop();
        assert_eq!(m.o, true);
    }

    #[test]
    fn bit_test_module_1() {
        let mut m = BitTestModule1::new();

        m.i = 0b0110;
        m.prop();
        assert_eq!(m.o0, false);
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);
        assert_eq!(m.o3, false);
    }

    #[test]
    fn bits_test_module_0() {
        let mut m = BitsTestModule0::new();

        m.i = 0b0110;
        m.prop();
        assert_eq!(m.o210, 0b110);
        assert_eq!(m.o321, 0b011);
        assert_eq!(m.o10, 0b10);
        assert_eq!(m.o32, 0b01);
        assert_eq!(m.o2, true);

        m.i = 0b1001;
        m.prop();
        assert_eq!(m.o210, 0b001);
        assert_eq!(m.o321, 0b100);
        assert_eq!(m.o10, 0b01);
        assert_eq!(m.o32, 0b10);
        assert_eq!(m.o2, false);

        m.i = 0b1111;
        m.prop();
        assert_eq!(m.o210, 0b111);
        assert_eq!(m.o321, 0b111);
        assert_eq!(m.o10, 0b11);
        assert_eq!(m.o32, 0b11);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn bits_test_module_1() {
        let mut m = BitsTestModule1::new();

        m.i = 0xfadebabedeadbeefabad1deabadc0de5;
        m.prop();
        assert_eq!(m.o0, 0xfadebabedeadbeefabad1deabadc0de5u128);
        assert_eq!(m.o1, 0x7adebabedeadbeefabad1deabadc0de5u128);
        assert_eq!(m.o2, 0xfadebabedeadbeefu64);
        assert_eq!(m.o3, 0xabad1deabadc0de5u64);
        assert_eq!(m.o4, 0xfadebabeu32);
        assert_eq!(m.o5, 0xdeadbeefu32);
        assert_eq!(m.o6, 0xabad1deau32);
        assert_eq!(m.o7, 0xbadc0de5u32);
        assert_eq!(m.o8, 0xadebabedeadbeefau64);
        assert_eq!(m.o9, true);
        assert_eq!(m.o10, 0xabadu32);
        assert_eq!(m.o11, true);
    }

    #[test]
    fn repeat_test_module() {
        let mut m = RepeatTestModule::new();

        m.i = 0xa;
        m.prop();
        assert_eq!(m.o0, 0xau32);
        assert_eq!(m.o1, 0xaau32);
        assert_eq!(m.o2, 0xaaaaau32);
        assert_eq!(m.o3, 0xaaaaaaaau32);
        assert_eq!(m.o4, 0xaaaaaaaaaaaaaaaau64);
        assert_eq!(m.o5, 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaau128);
        assert_eq!(m.o6, 0b000u32);
        assert_eq!(m.o7, 0x0u64);
        assert_eq!(m.o8, 0x0u128);

        m.i = 0x5;
        m.prop();
        assert_eq!(m.o0, 0x5u32);
        assert_eq!(m.o1, 0x55u32);
        assert_eq!(m.o2, 0x55555u32);
        assert_eq!(m.o3, 0x55555555u32);
        assert_eq!(m.o4, 0x5555555555555555u64);
        assert_eq!(m.o5, 0x55555555555555555555555555555555u128);
        assert_eq!(m.o6, 0b111u32);
        assert_eq!(m.o7, 0xffffffffffffffffu64);
        assert_eq!(m.o8, 0xffffffffffffffffffffffffffffffffu128);
    }

    #[test]
    fn concat_test_module() {
        let mut m = ConcatTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0x5;
        m.i3 = 0xfadebabe;
        m.prop();
        assert_eq!(m.o0, 0xa5u32);
        assert_eq!(m.o1, 0x5au32);
        assert_eq!(m.o2, 0xau32);
        assert_eq!(m.o3, 0x1au32);
        assert_eq!(m.o4, 0x1au32);
        assert_eq!(m.o5, 0xfadebabefadebabeu64);
        assert_eq!(m.o6, 0xfadebabefadebabefadebabefadebabeu128);
    }

    #[test]
    fn eq_test_module() {
        let mut m = EqTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xa;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn ne_test_module() {
        let mut m = NeTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xa;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, false);
    }

    #[test]
    fn lt_test_module() {
        let mut m = LtTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, false);
    }

    #[test]
    fn le_test_module() {
        let mut m = LeTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn gt_test_module() {
        let mut m = GtTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);
    }

    #[test]
    fn ge_test_module() {
        let mut m = GeTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn lt_signed_test_module() {
        let mut m = LtSignedTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, false);
    }

    #[test]
    fn le_signed_test_module() {
        let mut m = LeSignedTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, true);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, true);
        assert_eq!(m.o2, false);
    }

    #[test]
    fn gt_signed_test_module() {
        let mut m = GtSignedTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn ge_signed_test_module() {
        let mut m = GeSignedTestModule::new();

        m.i1 = 0xa;
        m.i2 = 0xb;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, false);

        m.i1 = 0b01;
        m.i2 = 0b11;
        m.prop();
        assert_eq!(m.o1, false);
        assert_eq!(m.o2, true);
    }

    #[test]
    fn mux_test_module() {
        let mut m = MuxTestModule::new();

        m.i1 = false;
        m.invert = false;
        m.prop();
        assert_eq!(m.o1, false);

        m.i1 = true;
        m.invert = false;
        m.prop();
        assert_eq!(m.o1, true);

        m.i1 = false;
        m.invert = true;
        m.prop();
        assert_eq!(m.o1, true);

        m.i1 = true;
        m.invert = true;
        m.prop();
        assert_eq!(m.o1, false);

        m.i2 = false;
        m.invert = false;
        m.prop();
        assert_eq!(m.o2, false);

        m.i2 = true;
        m.invert = false;
        m.prop();
        assert_eq!(m.o2, true);

        m.i2 = false;
        m.invert = true;
        m.prop();
        assert_eq!(m.o2, true);

        m.i2 = true;
        m.invert = true;
        m.prop();
        assert_eq!(m.o2, false);
    }

    #[test]
    fn instantiation_test_module_comb() {
        let mut m = InstantiationTestModuleComb::new();

        m.i1 = 0xffffffff;
        m.i2 = 0xffff0000;
        m.i3 = 0x00ff0000;
        m.i4 = 0x000f0000;
        m.prop();
        assert_eq!(m.o, 0x000f0000u32);

        m.i1 = 0x00000f00;
        m.i2 = 0xffffffff;
        m.i3 = 0x0000ffff;
        m.i4 = 0xffffffff;
        m.prop();
        assert_eq!(m.o, 0x00000f00u32);
    }

    #[test]
    fn instantiation_test_module_reg() {
        let mut m = InstantiationTestModuleReg::new();

        // Check initial value
        m.reset();
        m.prop();
        assert_eq!(m.o, 0);

        // The inputs propagate through 2 registers, so we won't see proper output for 2 cycles
        m.i1 = 0xffffffff;
        m.i2 = 0xffff0000;
        m.i3 = 0x00ff0000;
        m.i4 = 0x000f0000;
        m.prop(); // Propagate to first register inputs
        assert_eq!(m.o, 0u32);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0u32);
        m.posedge_clk();
        m.prop();
        assert_eq!(m.o, 0x000f0000u32);
    }

    #[test]
    fn nested_instantiation_test_module() {
        let mut m = NestedInstantiationTestModule::new();

        m.i1 = 0xffffffff;
        m.i2 = 0xffff0000;
        m.i3 = 0x00ff0000;
        m.i4 = 0x000f0000;
        m.prop();
        assert_eq!(m.o, 0x000f0000u32);

        m.i1 = 0x00000f00;
        m.i2 = 0xffffffff;
        m.i3 = 0x0000ffff;
        m.i4 = 0xffffffff;
        m.prop();
        assert_eq!(m.o, 0x00000f00u32);
    }
}
