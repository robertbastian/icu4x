import 'package:test/test.dart';
import 'package:icu/icu.dart';

void main() {
  test('Decimal.toString', () {
    final x = Decimal.fromDoubleWithLowerMagnitude(1.49403, -7);
    expect(x.toString(), '1.4940300');
  });
}
