# 1. Create or clear the file


echo "" >> cat_aimy.rs
echo "// The library:" > cat_aimy.rs
echo "" >> cat_aimy.rs
cat ./Cargo.toml >> cat_aimy.rs
# cat ../dsta/src/lib.rs >> cat_aimy.rs
cat ./src/main.rs >> cat_aimy.rs


echo "" >> cat_aimy.rs
echo "// The new controls:" >> cat_aimy.rs
echo "" >> cat_aimy.rs
find ./src/lens/v_stev/ -name "*.rs" -exec cat {} >> cat_aimy.rs \;
find ./src/form/v_nico/ -name "*.rs" -exec cat {} >> cat_aimy.rs \;


echo "" >> cat_aimy.rs
echo "// Building Example for LENS:" >> cat_aimy.rs
echo "" >> cat_aimy.rs
cat ../../allm/src/lib.rs >> cat_aimy.rs
cat ../../allm/src/client.rs >> cat_aimy.rs


echo "" >> cat_aimy.rs
echo "// Building Example for LENS:" >> cat_aimy.rs
echo "" >> cat_aimy.rs
cat ./src/lens/v_dr_r/state.rs >> cat_aimy.rs
cat ./src/lens/v_dr_r/view.rs >> cat_aimy.rs

echo "" >> cat_aimy.rs
cat ./src/lens/v_log_notes/state.rs >> cat_aimy.rs
cat ./src/lens/v_log_notes/view.rs >> cat_aimy.rs


echo "" >> cat_aimy.rs
echo "// Building Example for FORM:" >> cat_aimy.rs
echo "" >> cat_aimy.rs
find ./src/form/v_buzz_make/ -name "*.rs" -exec cat {} >> cat_aimy.rs \;
echo "" >> cat_aimy.rs
