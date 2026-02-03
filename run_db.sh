#!/bin/bash

surreal start --log warn --user root --pass root  -b 127.0.0.1:8083  rocksdb:./data/db
