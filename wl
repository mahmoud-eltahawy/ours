#!/usr/bin/env nu

const webls_root = '/home/eltahawy';
const webls_port = 3000;

def "main dev" [] {
  load-env {
  	"WEBLS_ROOT":$webls_root,
  	"WEBLS_PORT":$webls_port,
  }; cargo leptos watch
}

def "main prod" [] {
  load-env {
  	"WEBLS_ROOT":$webls_root,
  	"WEBLS_PORT":$webls_port,

  }; ./target/release/webls
}

def main [] {}
