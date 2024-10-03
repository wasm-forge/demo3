#!/bin/bash
source build.sh
source deploy.sh

source bench1.sh &> report/bench1.log
