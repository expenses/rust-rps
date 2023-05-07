(module
  (type (;0;) (func (param i32)))
  (type (;1;) (func (param i64 i32 i64 i32)))
  (type (;2;) (func (param i32 i32 i32 i32 i32 i32 i32)))
  (type (;3;) (func (param i32 i32 i64 i32 i32) (result i32)))
  (type (;4;) (func))
  (type (;5;) (func (param i32 i64 i32)))
  (type (;6;) (func (param i64 f32)))
  (import "env" "___rpsl_abort" (func (;0;) (type 0)))
  (import "env" "___rpsl_describe_handle" (func (;1;) (type 1)))
  (import "env" "___rpsl_block_marker" (func (;2;) (type 2)))
  (import "env" "___rpsl_node_call" (func (;3;) (type 3)))
  (func (;4;) (type 4)
    nop)
  (func (;5;) (type 5) (param i32 i64 i32)
    (local i64 i64 i64)
    global.get 0
    local.tee 3
    local.set 5
    block  ;; label = @1
      block  ;; label = @2
        local.get 0
        i32.const 2
        i32.eq
        if  ;; label = @3
          local.get 1
          i64.load
          local.set 4
          i32.const 1
          local.set 0
          local.get 3
          i64.const 48
          i64.sub
          local.tee 3
          global.set 0
          local.get 2
          i32.const 1
          i32.and
          i32.eqz
          br_if 1 (;@2;)
          local.get 4
          local.set 3
          br 2 (;@1;)
        end
        i32.const -3
        call 0
        local.get 5
        global.set 0
        return
      end
      local.get 4
      i32.load offset=24
      local.set 2
      local.get 4
      i32.load
      i32.const 4
      i32.ne
      if  ;; label = @2
        local.get 4
        i32.load offset=20
        local.set 0
      end
      local.get 3
      i64.const 0
      i64.store
      local.get 3
      i64.const 216736831578832896
      i64.store offset=28 align=4
      local.get 3
      local.get 0
      i32.store offset=24
      local.get 3
      i32.const 0
      i32.store offset=20
      local.get 3
      local.get 2
      i32.store16 offset=18
      local.get 3
      i64.const 16
      i64.add
      i32.const 0
      i32.store16
      local.get 3
      i64.const 8
      i64.add
      i64.const 0
      i64.store
    end
    local.get 3
    local.get 1
    i64.load offset=8
    f32.load
    call 6
    local.get 5
    global.set 0)
  (func (;6;) (type 6) (param i64 f32)
    (local i64 i64 i64 i32 i32 i32 f32 f32 f32)
    global.get 0
    i64.const 240
    i64.sub
    local.tee 4
    global.set 0
    local.get 4
    local.tee 2
    i64.const 26
    i64.add
    local.get 0
    i64.const 18
    i64.add
    i32.load16_u
    local.tee 6
    i32.store16
    local.get 2
    local.get 0
    i32.load
    local.tee 5
    i32.store offset=56
    local.get 2
    local.get 5
    i32.store offset=8
    local.get 2
    local.get 0
    i32.load offset=4
    local.tee 5
    i32.store offset=60
    local.get 2
    local.get 5
    i32.store offset=12
    local.get 2
    local.get 0
    i32.load offset=8
    local.tee 5
    i32.store offset=64
    local.get 2
    local.get 5
    i32.store offset=16
    local.get 2
    local.get 0
    i32.load offset=12
    local.tee 5
    i32.store offset=68
    local.get 2
    local.get 5
    i32.store offset=20
    local.get 2
    local.get 0
    i32.load16_u offset=16
    local.tee 5
    i32.store16 offset=72
    local.get 2
    local.get 5
    i32.store16 offset=24
    local.get 2
    local.get 6
    i32.store16 offset=74
    local.get 2
    local.get 2
    i64.const 8
    i64.add
    local.tee 3
    i64.store offset=48
    local.get 2
    i64.const 28
    i64.add
    local.get 0
    i64.const 20
    i64.add
    i32.load
    local.tee 6
    i32.store
    local.get 2
    i64.const 32
    i64.add
    local.get 0
    i64.const 24
    i64.add
    i32.load
    local.tee 5
    i32.store
    local.get 2
    local.get 6
    i32.store offset=76
    local.get 2
    local.get 5
    i32.store offset=80
    local.get 2
    local.get 0
    f32.load offset=28
    local.tee 8
    f32.store offset=84
    local.get 2
    local.get 8
    f32.store offset=36
    local.get 2
    local.get 0
    i32.load offset=32
    local.tee 5
    i32.store offset=88
    local.get 2
    local.get 5
    i32.store offset=40
    local.get 2
    i64.const 200
    i64.add
    i32.const 36
    local.get 3
    i32.const 1
    call 1
    local.get 2
    local.get 2
    i64.const 212
    i64.add
    i64.store offset=96
    local.get 2
    local.get 2
    i32.load offset=212
    local.tee 6
    i32.store offset=104
    block  ;; label = @1
      local.get 6
      i32.eqz
      if  ;; label = @2
        f32.const 0x1p+0 (;=1;)
        local.set 9
        br 1 (;@1;)
      end
      local.get 2
      local.get 2
      i32.load offset=216
      local.tee 5
      i32.store offset=108
      local.get 2
      local.get 5
      f32.convert_i32_u
      local.get 6
      f32.convert_i32_u
      f32.div
      local.tee 9
      f32.store offset=112
    end
    i32.const 1
    i32.const 0
    i32.const 0
    i32.const 1
    i32.const 0
    i32.const 0
    i32.const -1
    call 2
    loop  ;; label = @1
      i32.const 2
      i32.const 0
      i32.const 0
      i32.const 0
      i32.const 0
      i32.const 0
      i32.const 0
      call 2
      local.get 2
      i64.const 4575657221408423936
      i64.store offset=192
      local.get 2
      local.get 2
      i64.load offset=96
      i32.load
      local.tee 6
      i32.store offset=116
      local.get 2
      local.get 2
      i32.load offset=216
      local.tee 5
      i32.store offset=124
      local.get 2
      local.get 5
      f32.convert_i32_u
      f32.const 0x1.555556p-2 (;=0.333333;)
      f32.mul
      local.tee 10
      f32.store offset=128
      local.get 2
      local.get 10
      f32.store offset=188
      local.get 2
      local.get 6
      f32.convert_i32_u
      f32.const 0x1.555556p-2 (;=0.333333;)
      f32.mul
      local.tee 8
      f32.store offset=120
      local.get 2
      local.get 8
      f32.store offset=184
      local.get 2
      local.get 10
      local.get 7
      i32.const 3
      i32.div_u
      local.tee 5
      f32.convert_i32_u
      f32.mul
      f32.store offset=180
      local.get 2
      local.get 8
      local.get 7
      local.get 5
      i32.const -3
      i32.mul
      i32.add
      f32.convert_i32_u
      f32.mul
      f32.store offset=176
      local.get 4
      i64.const 32
      i64.sub
      local.tee 3
      local.tee 0
      global.set 0
      local.get 2
      local.get 3
      i64.store offset=144
      local.get 2
      local.get 3
      i64.store offset=136
      local.get 3
      local.get 2
      i64.const 8
      i64.add
      i64.store
      local.get 0
      i64.const 16
      i64.sub
      local.tee 0
      global.set 0
      local.get 0
      local.get 9
      f32.store
      local.get 3
      local.get 0
      i64.store offset=8
      local.get 2
      local.get 0
      i64.store offset=152
      local.get 0
      i64.const 16
      i64.sub
      local.tee 0
      local.tee 4
      global.set 0
      local.get 0
      local.get 1
      f32.store
      local.get 3
      local.get 0
      i64.store offset=16
      local.get 2
      local.get 0
      i64.store offset=160
      local.get 3
      local.get 2
      i64.const 176
      i64.add
      i64.store offset=24
      i32.const 0
      i32.const 4
      local.get 3
      i32.const 0
      i32.const 0
      call 3
      local.set 5
      local.get 2
      local.get 7
      i32.const 1
      i32.add
      local.tee 7
      i32.store offset=172
      local.get 2
      local.get 5
      i32.store offset=168
      local.get 7
      i32.const 10
      i32.ne
      br_if 0 (;@1;)
    end
    i32.const 3
    i32.const 0
    i32.const 0
    i32.const 1
    i32.const 0
    i32.const 0
    i32.const -1
    call 2
    local.get 2
    i64.const 240
    i64.add
    global.set 0)
  (table (;0;) 3 3 funcref)
  (memory (;0;) i64 2)
  (global (;0;) (mut i64) (i64.const 67680))
  (global (;1;) i64 (i64.const 1024))
  (global (;2;) i64 (i64.const 1464))
  (global (;3;) i64 (i64.const 1736))
  (global (;4;) i64 (i64.const 1832))
  (global (;5;) i64 (i64.const 1872))
  (global (;6;) i64 (i64.const 1880))
  (global (;7;) i64 (i64.const 1024))
  (global (;8;) i64 (i64.const 2136))
  (global (;9;) i64 (i64.const 1024))
  (global (;10;) i64 (i64.const 67680))
  (global (;11;) i64 (i64.const 131072))
  (global (;12;) i64 (i64.const 0))
  (global (;13;) i64 (i64.const 1))
  (global (;14;) i64 (i64.const 1))
  (export "memory" (memory 0))
  (export "__wasm_call_ctors" (func 4))
  (export "rpsl_M_rps_multithreading_Fn_main_wrapper" (func 5))
  (export "rpsl_M_rps_multithreading_Fn_main" (func 6))
  (export "___rpsl_string_table_rps_multithreading" (global 1))
  (export "___rpsl_module_info_rps_multithreading" (global 2))
  (export "NodeDecls_rps_multithreading" (global 3))
  (export "rpsl_M_rps_multithreading_E_main_AE_value" (global 4))
  (export "rpsl_M_rps_multithreading_E_main" (global 5))
  (export "rpsl_M_rps_multithreading_E_main_pp" (global 6))
  (export "__indirect_function_table" (table 0))
  (export "__dso_handle" (global 7))
  (export "__data_end" (global 8))
  (export "__global_base" (global 9))
  (export "__heap_base" (global 10))
  (export "__heap_end" (global 11))
  (export "__memory_base" (global 12))
  (export "__table_base" (global 13))
  (export "__table_base32" (global 14))
  (elem (;0;) (i32.const 1) func 6 5)
  (data (;0;) (i64.const 1024) "rps_multithreading\00renderTarget\00oneOverAspectRatio\00timeInSeconds\00viewport\00GeometryPass\00main\00backbuffer\00\00\00\00\00\00J\00\00\00\00\00\00\00\04\00\00\00\01")
  (data (;1;) (i64.const 1168) "\06\00\00\00\00\00\00\00$\00\00\00\04\00\00\00\04 \00\00\00\00\00\00\04\00\00\00\04\00\00\00\05\00\00\00\00\00\00\00\18\00\00\00\04")
  (data (;2;) (i64.const 1232) "\13\00\00\00\00\00\00\00\80\00\00\00\ff\ff\ff\ff\00\00\00\00$\00\00\00 \00\00\00\01\00\00\00\00\00\00\00\ff\ff\ff\ff\00\00\00\00\04\00$\003\00\00\00\01\00\00\00\00\00\00\00\ff\ff\ff\ff\00\00\00\00\04\00(\00A\00\00\00\02\00\00\00\00\00\00\00\ff\ff\ff\ff\00\00\00\00\18\00,\00\5c")
  (data (;3;) (i64.const 1338) "\08\00\ff\ff\ff\ff\00\00\00\00$\00\00\003\00\00\00\01\00\00\00\00\00\00\00\ff\ff\ff\ff\00\00\00\00\04\00$")
  (data (;4;) (i64.const 1404) "W\00\00\00\04\00\00\00\02\00\00\00\01\00\00\00\00\00\00\00\02")
  (data (;5;) (i64.const 1464) "RPSM\03\00\00\00\09\00\00\00\00\00\00\00g\00\00\00\01\00\00\00\03\00\00\00\06\00\00\00\01")
  (data (;6;) (i64.const 1517) "\04\00\00\00\00\00\00h\04\00\00\00\00\00\00\90\04\00\00\00\00\00\00\d0\04\00\00\00\00\00\00x\05\00\00\00\00\00\00\d8\07\00\00\00\00\00\00\e8\07\00\00\00\00\00\00\f8\07\00\00\00\00\00\00\18\08\00\00\00\00\00\00RPSM$\00@\00\00\00\00\00`\07\00\00\00\00\00\00p\07\00\00\00\00\00\00\04\00\00\00\00\00\00\00\04\00\00\00\00\00\00\00(\08\00\00\00\00\00\00}\07")
  (data (;7;) (i64.const 1656) "\04\00\00\00\00\00\00\008\08\00\00\00\00\00\00\90\07")
  (data (;8;) (i64.const 1688) "\18\00\00\00\00\00\00\00\a0\07\00\00\00\00\00\00\b0\07")
  (data (;9;) (i64.const 1720) "GeometryPass\00\00\00\00\01\00\00\00\04\00\00\008\06\00\00\00\00\00\00\b8\06\00\00\00\00\00\00main\00\00\00\00$\00@\00\00\00\00\00\bc\07\00\00\00\00\00\00\cc\07\00\00\00\00\00\00\04\00\00\00\00\00\00\00\04\00\00\00\00\00\00\00H\08\00\00\00\00\00\00\90\07")
  (data (;10;) (i64.const 1832) "\e0\06\00\00\00\00\00\00\02\00\00\00\00\00\00\00\e8\06\00\00\00\00\00\00\c8\06\00\00\00\00\00\00\02\00\00\00\01\00\00\00(\07\00\00\00\00\00\00P\07\00\00\00\00\00\00\80\00\00\00\00\00\00\00#\00\00\00\00\00\00\00renderTarget\00oneOverAspectRatio\00timeInSeconds")
  (data (;11;) (i64.const 1960) "\11\00\00\00\00\00\00\00viewport\00\00\00\00\00\00\08")
  (data (;12;) (i64.const 1996) "backbuffer"))
