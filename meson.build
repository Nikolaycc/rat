project('rat', ['c', 'rust'], version : '0.1.0')
rat_version = '0.1.0'

# include
rat_lib_inc = include_directories('rat_lib')

# rat_lib
# add_project_arguments('-DDEBUG', language: 'c') # DEBUG ENABLE
rat_lib = static_library('rat', 'rat_lib/rat.c', 'rat_lib/rat_packets.c', 'rat_lib/rat_utils.c', include_directories : rat_lib_inc)

# rat_cli
readline_dep = dependency('readline', required : false)
rat_cli = executable('rat_cli', 'rat_cli/rat_cli.c', c_args: '-DRAT_VERSION="v'+ rat_version +'"', link_with : rat_lib, dependencies : readline_dep, include_directories : rat_lib_inc, install : true)

# rat_rs
rat_rs = static_library('Rat', 'rat_rs/lib.rs', link_with : rat_lib, rust_crate_type : 'rlib')

# tests
executable('test_rat_cap', 'tests/test_rat_cap.c', link_with : rat_lib, include_directories : incdir)
executable('test_explore', 'tests/test_explore.c')
executable('test_rat_rust', 'tests/test_rat_rust.rs', link_with : rat_rs)
executable('test_rat_rust_iter', 'tests/test_rat_rust_iter.rs', link_with : rat_rs)
executable('test_rat_rust_packets', 'tests/test_rat_rust_packets.rs', link_with : rat_rs)

# test('test_rat_cap', test_rat_cap)
