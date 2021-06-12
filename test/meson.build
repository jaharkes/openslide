tests = [
  ['test_test', 'test.c', [glib2, cairo, libm]],
  ['test_try_open', 'try_open.c', [glib2]],
  ['test_parallel', 'parallel.c', [glib2]],
  ['test_query', 'query.c', [glib2]],
  ['test_extended', 'extended.c', [glib2]],
  ['test_mosaic', 'mosaic.c', [glib2, cairo]],
  ['test_profile', 'profile.c', [glib2]],
]

foreach t : tests
  exe = executable(t[0], t[1],
      dependencies : [libopenslide, libopenslide_common] + t[2] + [valgrind],
  )
  # the test executables don't actually run as standalone tests but are used by
  # the driver script
  # test(t[0], exe)
endforeach
