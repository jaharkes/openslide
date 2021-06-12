project('OpenSlide', 'c',
  version : '3.4.1',
  license : 'LGPL-2.1-only',
  default_options : [
    'warning_level=3',
    'default_library=shared',
    'c_std=c99',
  ]
)
# bug-report: openslide-users@lists.andrew.cmu.edu


# libtool versioning - this applies to libopenslide
#
# See http://sources.redhat.com/autobook/autobook/autobook_91.html#SEC91 for details
#
# - If interfaces have been changed or added, but binary compatibility
#   has been preserved, change:
#      CURRENT += 1
#      REVISION = 0
#      AGE += 1
# - If binary compatibility has been broken (eg removed or changed
#   interfaces), change:
#      CURRENT += 1
#      REVISION = 0
#      AGE = 0
# - If the interface is the same as the previous version, but bugs are
#   fixed, change:
#      REVISION += 1
#
lt_current = '4'
lt_revision = '1'
lt_age = '4'
lt_version = '@0@.@1@.@2@'.format(lt_current, lt_age, lt_revision)


conf = configuration_data()

# get version and suffixed-version
version = meson.project_version()
conf.set_quoted('VERSION', version)

version_suffix = get_option('version-suffix')
if version_suffix != ''
  conf.set_quoted('SUFFIXED_VERSION', '@0@-@1@'.format(version, version_suffix))
else
  conf.set_quoted('SUFFIXED_VERSION', version)
endif


# check for functions/types
compiler = meson.get_compiler('c')
host_os = host_machine.system()
platform_win32 = (host_os.startswith('mingw') or
                  host_os.startswith('cygwin') or
                  host_os.startswith('windows'))
platform_osx = host_os.startswith('darwin')


if platform_win32
# if host_os == 'mingw32' and build_os == 'cygwin'
#   conf.set_quoted(CYGWIN_CROSS_TEST, 'yes')
# endif
  if compiler.has_argument('-mpreferred-stack-boundary=2')
    add_project_arguments('-mpreferred-stack-boundary=2', language : 'c')
  endif
  if compiler.has_argument('-Wno-pedantic-ms-format')
    add_project_arguments('-Wno-pedantic-ms-format', language : 'c')
  endif
endif

add_project_arguments('-Wstrict-prototypes', language : 'c')
add_project_arguments('-Wmissing-prototypes', language : 'c')
add_project_arguments('-Wmissing-declarations', language : 'c')
add_project_arguments('-Wnested-externs', language : 'c')

add_project_arguments('-DG_DISABLE_SINGLE_INCLUDES', language : 'c')
add_project_arguments('-DGLIB_VERSION_MIN_REQUIRED=GLIB_VERSION_2_56', language : 'c')
add_project_arguments('-DGLIB_VERSION_MAX_ALLOWED=GLIB_VERSION_MIN_REQUIRED', language : 'c')
add_project_arguments('-fno-common', language : 'c')

if compiler.has_argument('-fvisibility=hidden')
  add_project_arguments('-fvisibility=hidden', language : 'c')
endif


# fseeko not found when compiling with -std=c99...
# https://stackoverflow.com/questions/40770673/implicit-declaration-of-function-fseeko
# using 200809L so that strdup is defined as well
add_project_arguments('-D_POSIX_C_SOURCE=200809L', language : 'c')

if compiler.has_function('fseeko', prefix : '#define _POSIX_C_SOURCE 200809L\n#include <stdio.h>')
  conf.set('HAVE_FSEEKO', 1)
endif

if compiler.sizeof('uintptr_t', prefix : '#include <stdint.h>') != -1
  conf.set('HAVE_UINTPTR_T', 1)
endif

if platform_win32
  conf.set_quoted('FOPEN_CLOEXEC_FLAG', 'M')
else
  code = '''
  #include <stdio.h>
  #include <unistd.h>
  #include <fcntl.h>

  int main(int argc, char **argv)
  {
      FILE *fp = fopen("/dev/null", "re");
      if (fp != NULL) {
          int fd = fileno(fp);
          if (fd != -1) {
              long ret = fcntl(fd, F_GETFD);
              if (ret != -1 && (ret & FD_CLOEXEC)) {
                  return 0;
              }
          }
      }
      return 1;
  }
  '''
  result = compiler.run(code, name : 'check fopen FD_CLOEXEC flag')
  if result.compiled() and result.returncode() == 0
    conf.set_quoted('FOPEN_CLOEXEC_FLAG', 'e')
  else
    conf.set_quoted('FOPEN_CLOEXEC_FLAG', '')
    conf.set('NONATOMIC_CLOEXEC', 1)
  endif
endif

if compiler.has_function('fcntl', prefix : '#include <unistd.h>\n#include <fcntl.h>')
  conf.set('HAVE_FCNTL', 1)
endif

if platform_win32 and compiler.has_function('_wfopen', prefix : '#include <stdio.h>')
  conf.set('HAVE__WFOPEN', 1)
endif

if platform_osx
  code = '''
  #include <libproc.h>

  int main(int argc, char **argv) {
      proc_pidfdinfo(0, 0, PROC_PIDFDKQUEUEINFO, NULL, 0);
  }
  '''
  if compiler.links(code, name : 'check for proc_pidfdinfo')
    conf.set('HAVE_PROC_PIDFDINFO', 1)
  endif
endif


needs_libm = not compiler.has_function('floor', prefix : '#include <math.h>')
libm = compiler.find_library('m', required : needs_libm)
assert(compiler.has_function('floor', prefix : '#include <math.h>', dependencies : libm))

zlib = dependency('zlib', required : false)
if not zlib.found()
  zlib = compiler.find_library('z', required : true)
  assert(compiler.has_function('inflate', dependencies : zlib))
endif

libjpeg = dependency('libjpeg', version : '>= 1.5.0', required : false)
if not libjpeg.found()
  # IJG libjpeg, or libjpeg-turbo < 1.5.0
  libjpeg = compiler.find_library('jpeg', required : true)
  assert(compiler.has_function('jpeg_CreateDecompress', dependencies : libjpeg))
endif

openjpeg  = dependency('libopenjp2', version : '> 2.1.0')
libpng    = dependency('libpng', version : '> 1.2')
libtiff   = dependency('libtiff-4')
glib2     = dependency('glib-2.0', version : '>= 2.56')
gthread2  = dependency('gthread-2.0')
gio2      = dependency('gio-2.0')
gobject2  = dependency('gobject-2.0')
gdkpixbuf = dependency('gdk-pixbuf-2.0', version : '>= 2.14')
cairo     = dependency('cairo', version : '>= 1.2')
libxml2   = dependency('libxml-2.0')
sqlite3   = dependency('sqlite3', version : '>= 3.6.20')

valgrind  = dependency('valgrind', required : false)
if valgrind.found()
  conf.set('HAVE_VALGRIND', 1)
endif

configure_file(output : 'config.h', configuration : conf)
add_project_arguments('-I@0@'.format(meson.build_root()), language : 'c')

subdir('src')
subdir('common')
subdir('tools')
subdir('test')
