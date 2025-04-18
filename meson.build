project('rat', 'c')

# rat_lib
add_project_arguments('-DDEBUG', language: 'c') # DEBUG ENABLE
rat_lib = static_library('rat_lib', 'rat_lib/rat.c', 'rat_lib/rat_packets.c', 'rat_lib/rat_utils.c')

test_rat_cap = executable('test_rat_cap', 'tests/test_rat_cap.c', link_with : rat_lib)
executable('test_explore', 'tests/test_explore.c')

# test('test_rat_cap', test_rat_cap)
