# loop 10 times and print the ID of this times loop using multithreading (parallel)
for i in {0..17}; do
    echo "Loop $i"
    # run the script in the background
    node 2_launchpad_auto.js $i &
done