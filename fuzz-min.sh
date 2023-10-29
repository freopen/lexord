rm -rf fuzz/tmp_fuzz_out fuzz/tmp_cmin fuzz/tmp_tmin
mkdir fuzz/tmp_fuzz_out fuzz/tmp_tmin
cp fuzz/tmp_fuzz_wd/default/queue/id* fuzz/tmp_fuzz_out
cargo afl cmin -i fuzz/tmp_fuzz_out -o fuzz/tmp_cmin target/debug/lexord-fuzz
for f in fuzz/tmp_cmin/*
do
    cargo afl tmin -i $f -o fuzz/tmp_tmin/$(basename $f) target/debug/lexord-fuzz
done
rm fuzz/corpus/*
for f in fuzz/tmp_tmin/*
do
    md5=$(md5sum $f | awk '{print $1}')
    mv $f fuzz/corpus/$md5
done
