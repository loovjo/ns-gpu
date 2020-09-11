rm image-dumps/*.png

./compile-shaders.sh && cargo run
if [ $? -eq 0 ]
then
    cd image-dumps
    files_in_order=$(ls -rt)
    convert -delay 12 -loop 0 $files_in_order animation.gif
    cd ..
else
    echo "Error :("
fi
