import { ChangeEvent, useCallback, useEffect, useMemo, useState } from 'react';
import axios from 'axios';

interface Props {
  auth: string;
}

interface AnchorOption {
  value: string;
  label: string;
}

interface CanvasResponse {
  status?: string;
  command?: {
    layer?: string;
    width?: number;
    height?: number;
    clear?: boolean;
  };
}

const defaultAnchorOptions: AnchorOption[] = [
  { value: 'top_left', label: 'Top left' },
  { value: 'top_center', label: 'Top center' },
  { value: 'top_right', label: 'Top right' },
  { value: 'center_left', label: 'Center left' },
  { value: 'center', label: 'Center' },
  { value: 'center_right', label: 'Center right' },
  { value: 'bottom_left', label: 'Bottom left' },
  { value: 'bottom_center', label: 'Bottom center' },
  { value: 'bottom_right', label: 'Bottom right' },
];

function bytesToBase64(bytes: Uint8ClampedArray): string {
  let binary = '';
  const chunkSize = 0x8000;
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    const chunk = bytes.subarray(offset, Math.min(offset + chunkSize, bytes.length));
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

function formatNumber(value: number): string {
  if (!Number.isFinite(value) || value < 0) return '—';
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`;
  }
  return value.toString();
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export default function Canvas({ auth }: Props) {
  const [clientId, setClientId] = useState('');
  const [layer, setLayer] = useState('gm_overlay');
  const [width, setWidth] = useState('');
  const [height, setHeight] = useState('');
  const [rgbaBase64, setRgbaBase64] = useState('');
  const [opacity, setOpacity] = useState('1');
  const [durationMs, setDurationMs] = useState('');
  const [scale, setScale] = useState('1');
  const [zIndex, setZIndex] = useState('10000');
  const [anchor, setAnchor] = useState('center');
  const [posX, setPosX] = useState('0.5');
  const [posY, setPosY] = useState('0.5');
  const [clearCanvas, setClearCanvas] = useState(false);
  const [imagePreview, setImagePreview] = useState('');
  const [imageName, setImageName] = useState('');
  const [processingImage, setProcessingImage] = useState(false);
  const [sending, setSending] = useState(false);
  const [status, setStatus] = useState('');
  const [statusLevel, setStatusLevel] = useState<'success' | 'warning' | 'error' | ''>('');
  const [anchors, setAnchors] = useState<AnchorOption[]>(defaultAnchorOptions);

  const requestConfig = useMemo(
    () => ({ headers: { Authorization: `Bearer ${auth}` } }),
    [auth]
  );

  useEffect(() => {
    let cancelled = false;
    axios
      .get<{ anchors?: string[] }>('/gm/canvas/anchors', requestConfig)
      .then(response => {
        if (cancelled) return;
        const payload = response.data?.anchors;
        if (!Array.isArray(payload) || payload.length === 0) return;
        const options = payload.map((value: string): AnchorOption => ({
          value,
          label: value
            .split('_')
            .map(segment => segment.charAt(0).toUpperCase() + segment.slice(1))
            .join(' '),
        }));
        setAnchors(options);
      })
      .catch(() => {
        // Ignore errors and keep default options
      });
    return () => {
      cancelled = true;
    };
  }, [requestConfig]);

  const parseNumber = useCallback((value: string, fallback: number) => {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  }, []);

  const onFileChange = useCallback(
    async (event: ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;
      setProcessingImage(true);
      setStatus('');
      setStatusLevel('');
      try {
        const dataUrl: string = await new Promise((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => {
            if (typeof reader.result === 'string') {
              resolve(reader.result);
            } else {
              reject(new Error('Unsupported file result'));
            }
          };
          reader.onerror = () => reject(new Error('Failed to read file'));
          reader.readAsDataURL(file);
        });

        const image: HTMLImageElement = await new Promise((resolve, reject) => {
          const img = new Image();
          img.onload = () => resolve(img);
          img.onerror = () => reject(new Error('Unable to decode image'));
          img.src = dataUrl;
        });

        const canvas = document.createElement('canvas');
        canvas.width = image.width;
        canvas.height = image.height;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          throw new Error('Canvas context unavailable');
        }
        ctx.drawImage(image, 0, 0);
        const imageData = ctx.getImageData(0, 0, image.width, image.height);
        const base64 = bytesToBase64(imageData.data);

        setWidth(String(image.width));
        setHeight(String(image.height));
        setRgbaBase64(base64);
        setImagePreview(dataUrl);
        setImageName(file.name);
      } catch (error: any) {
        setStatus(error?.message || 'Failed to process image');
        setStatusLevel('error');
        setImagePreview('');
        setImageName('');
        setRgbaBase64('');
      } finally {
        setProcessingImage(false);
      }
    },
    []
  );

  const pixelMetrics = useMemo(() => {
    const parsedWidth = parseNumber(width, 0);
    const parsedHeight = parseNumber(height, 0);
    if (parsedWidth <= 0 || parsedHeight <= 0) {
      return {
        pixels: '—',
        bytes: '—',
      };
    }
    const pixelCount = Math.trunc(parsedWidth) * Math.trunc(parsedHeight);
    const byteCount = pixelCount * 4;
    return {
      pixels: `${Math.trunc(parsedWidth)} × ${Math.trunc(parsedHeight)}`,
      bytes: formatNumber(byteCount),
    };
  }, [height, parseNumber, width]);

  const statusClass = statusLevel
    ? `status-message status-${statusLevel}`
    : 'status-message';

  const resetForm = () => {
    setClientId('');
    setLayer('gm_overlay');
    setWidth('');
    setHeight('');
    setRgbaBase64('');
    setOpacity('1');
    setDurationMs('');
    setScale('1');
    setZIndex('10000');
    setAnchor('center');
    setPosX('0.5');
    setPosY('0.5');
    setClearCanvas(false);
    setImagePreview('');
    setImageName('');
    setStatus('');
    setStatusLevel('');
  };

  const send = async () => {
    setStatus('');
    setStatusLevel('');

    if (!clientId.trim()) {
      setStatus('Client ID is required');
      setStatusLevel('warning');
      return;
    }

    const parsedWidth = parseNumber(width, 0);
    const parsedHeight = parseNumber(height, 0);
    if (parsedWidth <= 0 || parsedHeight <= 0) {
      setStatus('Width and height must be positive numbers');
      setStatusLevel('warning');
      return;
    }

    if (!clearCanvas && !rgbaBase64.trim()) {
      setStatus('Upload an image or enable "Clear existing layers"');
      setStatusLevel('warning');
      return;
    }

    const opacityValue = clamp(parseNumber(opacity, 1), 0, 1);
    const scaleValue = parseNumber(scale, 1);
    const zIndexValue = Math.trunc(parseNumber(zIndex, 10_000));
    const durationValue = durationMs.trim()
      ? Math.max(0, Math.trunc(parseNumber(durationMs, 0)))
      : undefined;
    const xValue = clamp(parseNumber(posX, 0.5), 0, 1);
    const yValue = clamp(parseNumber(posY, 0.5), 0, 1);
    const anchorValue = anchor || 'center';

    const instruction: Record<string, unknown> = {
      layer: layer.trim() || 'gm_overlay',
      width: Math.trunc(parsedWidth),
      height: Math.trunc(parsedHeight),
      rgbaBase64: rgbaBase64.trim(),
      opacity: opacityValue,
      scale: scaleValue > 0 ? scaleValue : 1,
      zIndex: Number.isFinite(zIndexValue) ? zIndexValue : 10_000,
      clear: clearCanvas,
      position: {
        x: xValue,
        y: yValue,
        anchor: anchorValue,
      },
    };

    if (durationValue !== undefined) {
      instruction.durationMs = durationValue;
    }

    setSending(true);
    try {
      const payload = {
        clientId: clientId.trim(),
        instruction,
      };
      const response = await axios.post<CanvasResponse>(
        '/gm/canvas/send',
        payload,
        requestConfig
      );
      const summary = response.data?.command;
      const clearFlag = summary?.clear || clearCanvas;
      const dimensions = summary?.width && summary?.height
        ? `${summary.width}×${summary.height}`
        : `${Math.trunc(parsedWidth)}×${Math.trunc(parsedHeight)}`;
      if (clearFlag && !rgbaBase64.trim()) {
        setStatus('Clear command queued successfully');
      } else {
        setStatus(`Canvas command queued (${dimensions})`);
      }
      setStatusLevel('success');
    } catch (error: any) {
      const message =
        error?.response?.data?.message ||
        error?.response?.data?.error ||
        error?.message ||
        'Failed to send canvas command';
      setStatus(message);
      setStatusLevel('error');
    } finally {
      setSending(false);
    }
  };

  return (
    <div>
      <section className="section">
        <div className="section-header">
          <h2>Canvas Broadcast</h2>
          <button
            className="button-secondary"
            onClick={resetForm}
            disabled={sending || processingImage}
          >
            Reset form
          </button>
        </div>
        <p className="section-description">
          Push overlay imagery to connected clients via the live transparent canvas.
        </p>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="canvas-client">Client ID</label>
            <input
              id="canvas-client"
              placeholder="Target client identifier"
              value={clientId}
              onChange={event => setClientId(event.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-layer">Layer</label>
            <input
              id="canvas-layer"
              placeholder="Layer name"
              value={layer}
              onChange={event => setLayer(event.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-opacity">Opacity (0-1)</label>
            <input
              id="canvas-opacity"
              value={opacity}
              onChange={event => setOpacity(event.target.value)}
              placeholder="1"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-duration">Duration (ms)</label>
            <input
              id="canvas-duration"
              value={durationMs}
              onChange={event => setDurationMs(event.target.value)}
              placeholder="Optional"
            />
          </div>
        </div>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="canvas-width">Width (px)</label>
            <input
              id="canvas-width"
              value={width}
              onChange={event => setWidth(event.target.value)}
              placeholder="Width in pixels"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-height">Height (px)</label>
            <input
              id="canvas-height"
              value={height}
              onChange={event => setHeight(event.target.value)}
              placeholder="Height in pixels"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-scale">Scale</label>
            <input
              id="canvas-scale"
              value={scale}
              onChange={event => setScale(event.target.value)}
              placeholder="1"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-z">Z Index</label>
            <input
              id="canvas-z"
              value={zIndex}
              onChange={event => setZIndex(event.target.value)}
              placeholder="10000"
            />
          </div>
        </div>
        <div className="form-grid">
          <div className="field">
            <label htmlFor="canvas-x">Position X (0-1)</label>
            <input
              id="canvas-x"
              value={posX}
              onChange={event => setPosX(event.target.value)}
              placeholder="0.5"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-y">Position Y (0-1)</label>
            <input
              id="canvas-y"
              value={posY}
              onChange={event => setPosY(event.target.value)}
              placeholder="0.5"
            />
          </div>
          <div className="field">
            <label htmlFor="canvas-anchor">Anchor</label>
            <select
              id="canvas-anchor"
              value={anchor}
              onChange={event => setAnchor(event.target.value)}
            >
              {anchors.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="field">
          <label htmlFor="canvas-base64">RGBA Base64</label>
          <textarea
            id="canvas-base64"
            rows={4}
            placeholder="Paste RGBA base64 or upload an image below"
            value={rgbaBase64}
            onChange={event => setRgbaBase64(event.target.value)}
          />
          <small className="muted">
            Provide raw RGBA pixels encoded with Base64. Uploading an image will convert it
            automatically.
          </small>
        </div>
        <div className="field">
          <label htmlFor="canvas-file">Image upload</label>
          <input
            id="canvas-file"
            type="file"
            accept="image/png,image/jpeg,image/webp,image/bmp,image/gif"
            onChange={onFileChange}
            style={{ padding: 0 }}
          />
          <small className="muted">
            Supported formats are converted to raw RGBA on upload.
          </small>
        </div>
        <div className="field" style={{ flexDirection: 'row', alignItems: 'center', gap: '8px' }}>
          <input
            id="canvas-clear"
            type="checkbox"
            checked={clearCanvas}
            onChange={event => setClearCanvas(event.target.checked)}
            style={{ width: 'auto', margin: 0 }}
          />
          <label htmlFor="canvas-clear" style={{ margin: 0 }}>
            Clear existing layers before drawing
          </label>
        </div>
        <div className="chips" style={{ marginTop: '12px' }}>
          <span className="chip">Pixels {pixelMetrics.pixels}</span>
          <span className="chip">RGBA bytes {pixelMetrics.bytes}</span>
          {rgbaBase64.trim() && (
            <span className="chip">Base64 length {formatNumber(rgbaBase64.length)}</span>
          )}
        </div>
        {imagePreview && (
          <div className="image-preview">
            <img src={imagePreview} alt={imageName || 'Canvas preview'} />
            <p className="image-preview-details">
              {imageName || 'Uploaded image'} · {pixelMetrics.pixels} px
            </p>
          </div>
        )}
        {processingImage && (
          <p className="status-message status-warning">Processing image…</p>
        )}
        <button onClick={send} disabled={sending || processingImage}>
          {sending ? 'Sending…' : 'Send canvas command'}
        </button>
        {status && <p className={statusClass}>{status}</p>}
      </section>
    </div>
  );
}
