#!/bin/bash

# Copyright (c) 2018-2022 The MobileCoin Foundation

set -e -x

if [[ "$EUID" -ne 0 ]]; then
	echo "Must be run as superuser" >&2
	exit 1
fi

# ############################################### #
# builder-install-sgx - Add SGX SDK and reinstall protobuf
# (Note(chris): I don't understand the protobuf part right now)
#
# Inspired by:
# https://github.com/sebva/docker-sgx
# Note: The example is FROM ubuntu:bionic, which is 18.04
# Note: Not just 'FROM'ing it because they make no maintenance promises
# ############################################### #

set -e
set -u

cd /tmp

# Install SGX Ubuntu/Debian Repo
# NB: When updating dependencies, please remember to update the instructions in BUILD.md as well
(
	. /etc/os-release

	curl -o sgx.bin "https://download.01.org/intel-sgx/sgx-linux/2.17/distro/ubuntu${VERSION_ID}-server/sgx_linux_x64_sdk_2.17.100.3.bin"

	echo "deb [arch=amd64 signed-by=/etc/apt/trusted.gpg.d/intel-sgx-archive-keyring.gpg] https://download.01.org/intel-sgx/sgx_repo/ubuntu/ ${UBUNTU_CODENAME} main" > /etc/apt/sources.list.d/intel-sgx.list
)

wget -O- https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | \
	gpg --dearmor > /etc/apt/trusted.gpg.d/intel-sgx-archive-keyring.gpg

# Actually install stuff
apt-get update
apt-get install -yq --no-install-recommends \
	ca-certificates \
	build-essential \
	ocaml \
	ocamlbuild \
	automake \
	autoconf \
	libtool \
	wget \
	python \
	libssl-dev \
	libcurl4-openssl-dev \
	protobuf-compiler \
	git \
	libprotobuf-dev \
	alien \
	cmake \
	debhelper \
	uuid-dev \
	libxml2-dev \
	libsgx-uae-service \
	sgx-aesm-service

# Install *after* pkg-config so that they get registered correctly.
# pkg-config gets pulled in transitively via build-essential
chmod +x ./sgx.bin
./sgx.bin --prefix=/opt/intel \
rm ./sgx.bin


# Update .bashrc to source sgxsdk
echo 'source /opt/intel/sgxsdk/environment' >> /root/.bashrc
