version := `grep ^version Cargo.toml| sed -r 's/.*"([0-9].*)".*/\1/g'`

default:
  @just --summary --unsorted

publish: clean
  @if [ "$(git rev-parse --abbrev-ref HEAD)" != "main" ]; then exit 1; fi
  git pull origin main
  git tag "v{{version}}" && git push origin "v{{version}}"
  git push origin main --tags
  cargo publish

clean:
  rm -rf target/
