# frozen_string_literal: true

begin
  require_relative 'aam__ruby'
rescue LoadError
  begin
    require_relative '../ext/aam_rs/target/release/libaam_ruby'
  rescue LoadError
    require_relative '../ext/aam_rs/target/debug/libaam_ruby'
  end
end

module AamRb
  module_function

  def split_aam(content)
    result = {}
    current_name = nil
    current_builder = nil

    content.each_line do |raw_line|
      line = raw_line.strip
      next if line.empty?

      header = parse_section_header(line)
      if header
        result[current_name] = current_builder if current_name && current_builder
        current_name = header
        current_builder = AAMBuilder.new
        next
      end

      next unless current_name && current_builder

      assignment = parse_assignment(line)
      current_builder.add_line(*assignment) if assignment
    end

    result[current_name] = current_builder if current_name && current_builder
    result
  end

  def parse_section_header(line)
    return nil unless line.start_with?('#')

    rest = line[1..].strip
    rest.end_with?('.aam') ? rest : nil
  end
  private_class_method :parse_section_header

  def parse_assignment(line)
    key, value = line.split('=', 2)
    return nil unless value

    key = key.strip
    return nil if key.empty?

    [key, value.strip]
  end
  private_class_method :parse_assignment
end

