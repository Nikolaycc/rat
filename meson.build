project('rat', ['c', 'rust'], version : '0.2.0', meson_version : '>=1.3.0')
rat_version = '0.2.0'

# ---  C  ---

debugc = get_option('debugc')

if debugc 
  add_project_arguments('-DDEBUG', language: 'c')
endif

system = host_machine.system()
if system == 'windows'
  error('Unsupported platform')
endif

rat_lib_inc = include_directories('src')
rat_lib_src = [
  'src/rat_core.c', 'src/rat_packets.c', 'src/rat_utils.c'
]

if system in ['freebsd', 'openbsd', 'netbsd', 'dragonfly']
  rat_lib_src += ['src/platforms/rat_bsd.c']
elif system == 'linux'
  rat_lib_src += ['src/platforms/rat_linux.c']
elif system == 'darwin'
  rat_lib_src += ['src/platforms/rat_bsd.c']
endif

rat_lib = static_library('rat', rat_lib_src,
                         include_directories : rat_lib_inc,
                         c_args: ['-fvisibility=hidden']
                        )

build_rat_cli = get_option('build_rat_cli')

if build_rat_cli
  readline_dep = dependency('readline', required : true)
  libuv_dep = dependency('libuv', required : true)

  executable('rat_cli', 'cli/rat_cli.c',
             c_args: '-DRAT_VERSION="v'+ rat_version +'"',
             link_with : rat_lib,
             dependencies : [readline_dep, libuv_dep],
             include_directories : rat_lib_inc,
             install : true
            )
endif

allbuild = get_option('allbuild')

if allbuild
  # ---  RUST  ---

  rat_rs = static_library('Rat', 'rust/rat_rs/src/lib.rs', link_with : rat_lib, rust_abi : 'rust')

  # --- TESTS ---

  executable('test_rat_cap', 'tests/test_rat_cap.c', link_with : rat_lib, include_directories : rat_lib_inc)
  # executable('test_explore', 'tests/test_explore.c')
  executable('test_rat_rust', 'tests/test_rat_rust.rs', link_with : rat_rs)
  executable('test_rat_rust_iter', 'tests/test_rat_rust_iter.rs', link_with : rat_rs)
  executable('test_rat_rust_packets', 'tests/test_rat_rust_packets.rs', link_with : rat_rs)
endif
