pkgname=forge
pkgver=0.1.0
pkgrel=1
pkgdesc='Lightweight Linux process manager for self-hosted applications'
arch=('x86_64' 'aarch64')
url='https://github.com/lvender-12/Forge'
license=('unknown')
depends=('glibc')
makedepends=('cargo')
source=("git+${url}.git")
sha256sums=('SKIP')

pkgver() {
  cd "$srcdir/Forge"
  grep -m1 '^version' Cargo.toml | cut -d '"' -f 2
}

build() {
  cd "$srcdir/Forge"
  cargo build --release --locked --bin forge --bin forged
}

package() {
  cd "$srcdir/Forge"
  install -Dm755 target/release/forge "$pkgdir/usr/bin/forge"
  install -Dm755 target/release/forged "$pkgdir/usr/bin/forged"
}
