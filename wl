#!/usr/bin/env nu

const webls_root = '/home/eltahawy';

def "main dev" [] {
  load-env {
  	"WEBLS_ROOT":$webls_root,
  }; cargo leptos watch
}

def "main prod" [] {
  load-env {
  	"WEBLS_ROOT":$webls_root,
  }; ./target/release/webls
}

def main [] {}
