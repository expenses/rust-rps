./RenderPipelineShaders/tools/rps_hlslc/linux-x64/bin/rps-hlslc $1 && \
gcc $(basename $1).g.c -c -I RenderPipelineShaders/include/ -o $(basename $1 ".rpsl").o && \
ar rcs $(dirname $1)/lib$(basename $1 ".rpsl").a $(basename $1 ".rpsl").o && \
rm $(basename $1 ".rpsl").tmp.rps.ll $(basename $1).g.c $(basename $1 ".rpsl").o
