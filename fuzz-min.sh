rm -rf fuzz/tmp_fuzz_out fuzz/tmp_cmin fuzz/tmp_tmin
mkdir fuzz/tmp_fuzz_out fuzz/tmp_tmin
for f in fuzz/tmp_fuzz_wd/*/queue/id*
do
    md5=$(md5sum $f | awk '{print $1}')
    cp -f $f fuzz/tmp_fuzz_out/$md5
done
cargo afl cmin -T all -i fuzz/tmp_fuzz_out -o fuzz/tmp_cmin target/release/lexord_fuzz
ls fuzz/tmp_cmin | xargs -P $(nproc) -I % cargo afl tmin -i fuzz/tmp_cmin/% -o fuzz/tmp_tmin/% target/release/lexord_fuzz
rm fuzz/corpus/*
for f in fuzz/tmp_tmin/*
do
    md5=$(md5sum $f | awk '{print $1}')
    mv $f fuzz/corpus/$md5
done
