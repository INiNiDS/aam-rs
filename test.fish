#!/usr/bin/fish

set -x RUSTFLAGS "-A warnings"

set standalone_features 64bit hash-fx hash-ahash hash-ripemd perf-hash hash-rapidhash hash-std parallel

set aot_modes dev release unsafe_fast_path

echo (set_color --bold cyan)"[󰣇] Запуск тотальной проверки aam-rs..."(set_color normal)

echo (set_color yellow)"\n--- Стадия 1: Тестирование отдельных фич ---"(set_color normal)
for feat in $standalone_features
    echo -n "Testing feature [$feat]... "
    set -l extra_flags ""

    if string match -q "*hash*" $feat
        set extra_flags "--no-default-features"
    end

    if cargo test --quiet $extra_flags --features "$feat,aot,release" > /dev/null 2>&1
        echo (set_color green)"OK"(set_color normal)
    else
        echo (set_color red)"FAILED (check Cargo.toml for $feat)"(set_color normal)
    end
end

for mode in $aot_modes
    echo -n "Testing AOT mode [$mode]... "

    if cargo test --quiet --no-default-features --features "aot,$mode,hash-std" > /dev/null 2>&1
        echo (set_color green)"OK"(set_color normal)
    else
        echo (set_color red)"FAILED"(set_color normal)
    end
end

echo (set_color yellow)"\n--- Стадия 2: Standard Stress (30M keys) ---"(set_color normal)
# Используем существующие хэши
set hash_strategies hash-fx hash-ahash hash-rapidhash hash-std hash-ripemd

for hash in $hash_strategies
    echo (set_color blue)"\n[Benchmark] Strategy: $hash"(set_color normal)

    # Очистка старых бинарников если нужно
    rm -f generated_stress_test.aam.bin 2>/dev/null

    # Запуск примера. Добавлен --no-default-features для чистоты эксперимента
    if cargo run --quiet --example standard_stress --release --no-default-features --features aot,unsafe_fast_path,64bit,$hash,parallel
        echo (set_color green)"Success: $hash"(set_color normal)
    else
        echo (set_color red)"Ошибка при выполнении $hash"(set_color normal)
    end
end

echo (set_color --bold green)"\n[✓] Все операции завершены, Ваше Величество."(set_color normal)