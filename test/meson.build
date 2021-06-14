tests = [
  ['test_test', 'test.c', [glib2_dep, cairo_dep, libm_dep]],
  ['test_try_open', 'try_open.c', [glib2_dep]],
  ['test_parallel', 'parallel.c', [glib2_dep]],
  ['test_query', 'query.c', [glib2_dep]],
  ['test_extended', 'extended.c', [glib2_dep]],
  ['test_mosaic', 'mosaic.c', [glib2_dep, cairo_dep]],
  ['test_profile', 'profile.c', [glib2_dep]],
]

foreach t : tests
  exe = executable(t[0], t[1],
    include_directories : config_h_include,
    dependencies : [
      openslide_dep,
      openslide_common_dep,
      t[2],
      valgrind,
    ],
  )
  # the test executables don't actually run as standalone tests but are used by
  # the driver script
  # test(t[0], exe)
endforeach
