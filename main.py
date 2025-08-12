import base64
import sys

with open('out0.txt', 'r') as f:
    data = f.read().strip()
try:
    decoded = base64.b64decode(data, validate=False)
    with open('output', 'wb') as f:
        f.write(decoded)
except:
    print('Произошла ошибка декодирования', file=sys.stderr)
    sys.exit(1)