project('rat', ['c', 'rust'], version : '0.1.0')
rat_version = '0.1.0'

# ---  C  ---

# add_global_arguments('-DDEBUG', language : 'c')

rat_lib_inc = include_directories('c_src/rat_lib')
rat_lib_src = [
  'c_src/rat_lib/rat.c', 'c_src/rat_lib/rat_packets.c', 'c_src/rat_lib/rat_utils.c'
]

rat_lib = static_library('rat', rat_lib_src, include_directories : rat_lib_inc)

allbuild = get_option('allbuild')

if allbuild
  readline_dep = dependency('readline', required : false)
  rat_cli = executable('rat_cli', 'c_src/rat_cli/rat_cli.c', c_args: '-DRAT_VERSION="v'+ rat_version +'"', link_with : rat_lib, dependencies : readline_dep, include_directories : rat_lib_inc, install : true)

  # ---  RUST  ---

  rat_rs = static_library('Rat', 'crates/rat_rs/src/lib.rs', link_with : rat_lib, rust_crate_type : 'rlib')

  # --- TESTS ---

  executable('test_rat_cap', 'tests/test_rat_cap.c', link_with : rat_lib, include_directories : rat_lib_inc)
  executable('test_explore', 'tests/test_explore.c')
  executable('test_rat_rust', 'tests/test_rat_rust.rs', link_with : rat_rs)
  executable('test_rat_rust_iter', 'tests/test_rat_rust_iter.rs', link_with : rat_rs)
  executable('test_rat_rust_packets', 'tests/test_rat_rust_packets.rs', link_with : rat_rs)
endif
