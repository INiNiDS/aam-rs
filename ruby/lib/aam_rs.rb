# frozen_string_literal: true

begin
  require_relative '../ext/aam_rs/target/release/libaam_rs_ruby'
rescue LoadError
  require_relative '../ext/aam_rs/target/debug/libaam_rs_ruby'
end

