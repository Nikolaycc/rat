project('rat', 'c', version : '0.1.0')
rat_version = '0.1.0'

# include
incdir = include_directories('include')

# rat_lib
add_project_arguments('-DDEBUG', language: 'c') # DEBUG ENABLE
rat_lib = static_library('rat_lib', 'rat_lib/rat.c', 'rat_lib/rat_packets.c', 'rat_lib/rat_utils.c', include_directories : incdir)

# rat_cli
readline_dep = dependency('readline', required : false)
rat_cli = executable('rat_cli', 'rat_cli/rat_cli.c', c_args: '-DRAT_VERSION="v'+ rat_version +'"', link_with : rat_lib, dependencies : readline_dep, include_directories : incdir, install : true)

test_rat_cap = executable('test_rat_cap', 'tests/test_rat_cap.c', link_with : rat_lib, include_directories : incdir)
executable('test_explore', 'tests/test_explore.c')

# test('test_rat_cap', test_rat_cap)
