project('fretboard',
  'rust',
  version: '4.0',
  license: 'GPL-3.0-or-later',
  meson_version: '>= 0.62.0',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'dev.bragefuglseth.Fretboard'

version = meson.project_version()

prefix = get_option('prefix')
localedir = prefix / get_option('localedir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  application_id = base_id
endif

meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_source_root(),
  meson.project_build_root() / 'meson-dist' / meson.project_name() + '-' + meson.project_version(),
)

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
