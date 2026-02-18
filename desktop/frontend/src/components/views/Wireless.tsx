import { useState } from 'react';
import { useApp } from '../../context/AppContext';
import { api } from '../../services/api';
import styles from './Wireless.module.css';

export default function WirelessView() {
  const { deviceStatus, refreshDevice } = useApp();

  const savedEndpoint = deviceStatus?.wireless?.saved_endpoint ?? '';
  const autoReconnect = deviceStatus?.wireless?.auto_reconnect_enabled ?? false;

  const [endpoint, setEndpoint] = useState(savedEndpoint);
  const [status, setStatus] = useState<{ msg: string; kind: 'ok' | 'err' | 'info' } | null>(null);
  const [loading, setLoading] = useState(false);
  const [scanned, setScanned] = useState<string[]>([]);

  const run = async (label: string, fn: () => Promise<void>) => {
    setLoading(true);
    setStatus({ msg: label, kind: 'info' });
    try {
      await fn();
    } catch (err: any) {
      setStatus({ msg: err.message, kind: 'err' });
    } finally {
      setLoading(false);
    }
  };

  const handleSetupUsb = () => run('Enabling TCP/IP mode via USB…', async () => {
    const result = await api.wirelessEnableTcpip();
    if (result?.ip_address) {
      setEndpoint(result.ip_address);
      setStatus({ msg: `Ready — unplug USB then click Connect`, kind: 'ok' });
    } else {
      setStatus({ msg: result?.message || 'TCP/IP enabled, IP not detected', kind: 'info' });
    }
  });

  const handleScan = () => run('Scanning network for ADB devices…', async () => {
    const result = await api.wirelessScan();
    setScanned(result?.devices ?? []);
    setStatus({ msg: result?.message || 'Scan complete', kind: result?.devices?.length ? 'ok' : 'info' });
  });

  const handleConnect = () => run(`Connecting to ${endpoint}…`, async () => {
    const result = await api.wirelessConnect(endpoint, true);
    setStatus({ msg: result?.connected ? `Connected to ${endpoint}` : (result?.message || 'Connection failed'), kind: result?.connected ? 'ok' : 'err' });
    if (result?.connected) refreshDevice();
  });

  const handleDisconnect = () => run('Disconnecting…', async () => {
    const result = await api.wirelessDisconnect(endpoint || undefined);
    setStatus({ msg: result?.message || 'Disconnected', kind: 'ok' });
    refreshDevice();
  });

  const handleReconnect = () => run('Reconnecting…', async () => {
    const result = await api.wirelessReconnect();
    setStatus({ msg: result?.reconnected ? `Reconnected to ${result.endpoint}` : (result?.message || 'Reconnect failed'), kind: result?.reconnected ? 'ok' : 'err' });
    if (result?.reconnected) refreshDevice();
  });

  const isConnected = deviceStatus?.devices?.some(
    (d: any) => d.serial === endpoint && d.state === 'device'
  );

  return (
    <section className="content-view active panel" id="wireless-view">
      <div className="panel-heading">
        <div>
          <h2 className="ops-title">Wireless ADB</h2>
          <p className="panel-subtitle">Connect to your Quest over Wi-Fi without a USB cable</p>
        </div>
        {endpoint && (
          <span className={`${styles['wireless-connection-badge']} ${isConnected ? styles['badge-ok'] : styles['badge-idle']}`}>
            {isConnected ? `Connected · ${endpoint}` : endpoint}
          </span>
        )}
      </div>

      {status && (
        <div className={`${styles['wireless-status-bar']} ${styles[`wireless-status-${status.kind}`]}`}>
          {status.msg}
        </div>
      )}

      <div className={styles['wireless-grid']}>
        <div className={styles['wireless-section']}>
          <h3 className="diagnostics-section-title">Setup</h3>
          <p className="diagnostics-section-hint">
            Plug in via USB first, then click <strong>Setup via USB</strong> to enable wireless mode and auto-detect your IP address.
          </p>
          <div className="actions">
            <button type="button" className="btn-primary" onClick={handleSetupUsb} disabled={loading}>
              Setup via USB
            </button>
            <button type="button" className="btn-secondary" onClick={handleScan} disabled={loading}>
              Scan Network
            </button>
          </div>

          {scanned.length > 0 && (
            <div className={styles['wireless-scan-results']}>
              <span className={styles['wireless-scan-label']}>Found devices — click to select:</span>
              {scanned.map(ip => (
                <button
                  key={ip}
                  type="button"
                  className={`${styles['wireless-scan-item']}${endpoint === ip ? ` ${styles['active']}` : ''}`}
                  onClick={() => { setEndpoint(ip); setScanned([]); }}
                >
                  {ip}
                </button>
              ))}
            </div>
          )}
        </div>

        <div className={styles['wireless-section']}>
          <h3 className="diagnostics-section-title">Connect</h3>
          <p className="diagnostics-section-hint">Enter an IP:port manually or pick one from a scan above.</p>
          <input
            type="text"
            className={styles['wireless-endpoint-input']}
            placeholder="192.168.1.20:5555"
            value={endpoint}
            onChange={e => setEndpoint(e.target.value)}
            disabled={loading}
          />
          <div className="actions">
            <button type="button" className="btn-primary" onClick={handleConnect} disabled={loading || !endpoint}>
              Connect
            </button>
            <button type="button" className="btn-secondary" onClick={handleDisconnect} disabled={loading}>
              Disconnect
            </button>
          </div>
          <div className="actions" style={{ marginTop: 0 }}>
            <button type="button" className="btn-secondary" style={{ flex: 1 }} onClick={handleReconnect} disabled={loading}>
              Reconnect to Saved
            </button>
          </div>

          <label className={styles['wireless-auto-label']}>
            <input
              type="checkbox"
              checked={autoReconnect}
              onChange={async e => {
                if (e.target.checked && endpoint) {
                  await api.wirelessConnect(endpoint, true).catch(() => {});
                }
              }}
            />
            Auto-reconnect on launch
          </label>
        </div>
      </div>

      <div className={styles['wireless-howto']}>
        <h3 className="diagnostics-section-title">How it works</h3>
        <ol className={styles['wireless-steps']}>
          <li><span className={styles['wireless-step-num']}>1</span> Connect your Quest via USB cable</li>
          <li><span className={styles['wireless-step-num']}>2</span> Click <strong>Setup via USB</strong> — this puts ADB into TCP mode and reads your device IP</li>
          <li><span className={styles['wireless-step-num']}>3</span> Unplug the cable</li>
          <li><span className={styles['wireless-step-num']}>4</span> Click <strong>Connect</strong></li>
          <li><span className={styles['wireless-step-num']}>5</span> Enable <strong>Auto-reconnect</strong> so it connects automatically next time</li>
        </ol>
      </div>
    </section>
  );
}
