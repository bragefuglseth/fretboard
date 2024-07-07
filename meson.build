project('fretboard',
  'rust',
  version: '7.1',
  license: 'GPL-3.0-or-later',
  meson_version: '>= 0.62.0',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'dev.bragefuglseth.Fretboard'

version = meson.project_version()

dependency('gio-2.0', version: '>= 2.76')
dependency('glib-2.0', version: '>= 2.76')
dependency('gtk4', version: '>= 4.10.1')
dependency('libadwaita-1', version: '>= 1.5.alpha')

prefix = get_option('prefix')
localedir = prefix / get_option('localedir')
datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

if get_option('profile') == 'development'
  profile = 'Devel'
  app_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  app_id = base_id
endif

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
     glib_compile_schemas: true,
    gtk_update_icon_cache: true,
  update_desktop_database: true,
)
