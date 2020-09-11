rm image-dumps/*.png

./compile-shaders.sh && cargo run
if [ $? -eq 0 ]
then
    echo 'Encoding gif'
    cd image-dumps
    rm animation.gif
    files_in_order=$(ls -rt)
    convert -delay 3 -loop 0 $files_in_order animation.gif
    cd ..
else
    echo "Error :("
fi
