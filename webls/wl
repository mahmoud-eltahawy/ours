#!/usr/bin/env nu

const webls_root = '/home/eltahawy';
const webls_port = 3000;
const webls_password = '/home/eltahawy/magit/webls/webls_password.txt';

def "main dev" [] {
  load-env {
  	"WEBLS_ROOT":$webls_root,
  	"WEBLS_PORT":$webls_port,
  	"WEBLS_PASSWORD":$webls_password,
  }; cargo leptos watch
}

def main [] {}
