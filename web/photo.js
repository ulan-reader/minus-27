// Выбор фото еды и сжатие до JPEG перед отправкой ИИ (см. src/js.rs).
// Большие фото с телефона весят мегабайты; 1024 px хватает, чтобы узнать еду.

export function pickPhoto(maxSide) {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    input.onchange = async () => {
      const file = input.files && input.files[0];
      if (!file) return resolve(null);
      try {
        resolve(await toJpeg(file, maxSide));
      } catch {
        resolve(null);
      }
    };
    input.click();
  });
}

async function toJpeg(file, maxSide) {
  const url = URL.createObjectURL(file);
  try {
    const img = await new Promise((ok, fail) => {
      const i = new Image();
      i.onload = () => ok(i);
      i.onerror = fail;
      i.src = url;
    });
    const k = Math.min(1, maxSide / Math.max(img.naturalWidth, img.naturalHeight));
    const canvas = document.createElement('canvas');
    canvas.width = Math.round(img.naturalWidth * k);
    canvas.height = Math.round(img.naturalHeight * k);
    canvas.getContext('2d').drawImage(img, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL('image/jpeg', 0.82).split(',')[1];
  } finally {
    URL.revokeObjectURL(url);
  }
}
