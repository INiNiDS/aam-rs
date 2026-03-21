# frozen_string_literal: true

require 'minitest/autorun'
require_relative '../lib/aam_rs'

class AamRsTest < Minitest::Test
  def test_parse_find_obj
    value = AamRs.parse_find_obj("host = localhost\nport = 8080", 'host')
    assert_equal 'localhost', value
  end

  def test_parse_find_obj_reverse_lookup
    key = AamRs.parse_find_obj("host = localhost", 'localhost')
    assert_equal 'host', key
  end

  def test_parse_find_obj_returns_nil_for_unknown_key
    value = AamRs.parse_find_obj("host = localhost", 'missing')
    assert_nil value
  end

  def test_parse_find_obj_raises_on_invalid_content
    assert_raises(RuntimeError) do
      AamRs.parse_find_obj('invalid_line_without_equals', 'host')
    end
  end
end

