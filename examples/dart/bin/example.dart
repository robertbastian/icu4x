import 'package:icu/icu.dart';
import 'dart:io';

int main() {
  final alphabetic = CodePointSetData.alphabetic();

  while (true) {
    stdout.write("Character: ");
    final input = stdin.readLineSync();
    if (input == null) {
      break;
    }
    print('Alphabetic: ${alphabetic.contains(input.runes.first)}');
  }

  return 0;
}
