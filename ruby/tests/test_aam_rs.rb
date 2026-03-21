# frozen_string_literal: true

require 'minitest/autorun'
require_relative '../lib/aam_rs'

class AamRsTest < Minitest::Test
  def test_parse_find_obj
    value = AamRs.parse_find_obj("host = localhost\nport = 8080", 'host')
    assert_equal 'localhost', value
  end
end

