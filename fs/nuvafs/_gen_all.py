import pathlib, base64
B = pathlib.Path('F:/nuva-os/nuva/fs/nuvafs')
def w(n,b):(B/n).write_text(base64.b64decode(b).decode(),encoding='utf-8');print(f'{n}')

