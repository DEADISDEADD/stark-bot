import { useState, useEffect, FormEvent } from 'react';
import { Key, Trash2, Plus, ExternalLink, Check, X, Cloud, Upload, Download, Shield, AlertCircle, CheckCircle, Eye } from 'lucide-react';
import Card, { CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import Input from '@/components/ui/Input';
import { getApiKeys, upsertApiKey, deleteApiKey, getServiceConfigs, ApiKey, ServiceConfig, backupKeysToCloud, restoreKeysFromCloud, previewKeysFromCloud, CloudKeyPreview } from '@/lib/api';

export default function ApiKeys() {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [serviceConfigs, setServiceConfigs] = useState<ServiceConfig[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [savingKeys, setSavingKeys] = useState<Set<string>>(new Set());
  const [keyInputs, setKeyInputs] = useState<Record<string, string>>({});
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  // Cloud backup state
  const [isUploading, setIsUploading] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [isPreviewing, setIsPreviewing] = useState(false);
  const [backupMessage, setBackupMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [previewModalOpen, setPreviewModalOpen] = useState(false);
  const [previewKeys, setPreviewKeys] = useState<CloudKeyPreview[]>([]);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [keysData, configsData] = await Promise.all([
        getApiKeys(),
        getServiceConfigs(),
      ]);
      setKeys(keysData);
      setServiceConfigs(configsData);
    } catch (err) {
      setMessage({ type: 'error', text: 'Failed to load API keys' });
    } finally {
      setIsLoading(false);
    }
  };

  const loadKeys = async () => {
    try {
      const data = await getApiKeys();
      setKeys(data);
    } catch (err) {
      setMessage({ type: 'error', text: 'Failed to load API keys' });
    }
  };

  // Check if a key is configured
  const isKeyConfigured = (keyName: string): boolean => {
    return keys.some(k => k.key_name === keyName);
  };

  // Get configured key by name
  const getConfiguredKey = (keyName: string): ApiKey | undefined => {
    return keys.find(k => k.key_name === keyName);
  };

  // Check if all keys in a group are configured
  const isGroupComplete = (config: ServiceConfig): boolean => {
    return config.keys.every(k => isKeyConfigured(k.name));
  };

  // Check if any keys in a group are configured
  const isGroupPartial = (config: ServiceConfig): boolean => {
    const configured = config.keys.filter(k => isKeyConfigured(k.name));
    return configured.length > 0 && configured.length < config.keys.length;
  };

  const handleSaveKey = async (keyName: string) => {
    const value = keyInputs[keyName]?.trim();
    if (!value) {
      setMessage({ type: 'error', text: 'Please enter a value' });
      return;
    }

    setSavingKeys(prev => new Set(prev).add(keyName));
    setMessage(null);

    try {
      await upsertApiKey(keyName, value);
      setMessage({ type: 'success', text: 'API key saved successfully' });
      setKeyInputs(prev => ({ ...prev, [keyName]: '' }));
      await loadKeys();
    } catch (err) {
      setMessage({ type: 'error', text: 'Failed to save API key' });
    } finally {
      setSavingKeys(prev => {
        const next = new Set(prev);
        next.delete(keyName);
        return next;
      });
    }
  };

  const handleSaveGroup = async (config: ServiceConfig, e: FormEvent) => {
    e.preventDefault();

    // Validate all keys in the group have values
    const keysToSave = config.keys.filter(k => keyInputs[k.name]?.trim());
    if (keysToSave.length === 0) {
      setMessage({ type: 'error', text: 'Please enter at least one key' });
      return;
    }

    setMessage(null);

    // Save all keys that have values
    for (const key of keysToSave) {
      setSavingKeys(prev => new Set(prev).add(key.name));
    }

    try {
      for (const key of keysToSave) {
        await upsertApiKey(key.name, keyInputs[key.name].trim());
      }
      setMessage({ type: 'success', text: `${config.label} keys saved successfully` });

      // Clear inputs for saved keys
      const clearedInputs = { ...keyInputs };
      for (const key of keysToSave) {
        clearedInputs[key.name] = '';
      }
      setKeyInputs(clearedInputs);

      await loadKeys();
    } catch (err) {
      setMessage({ type: 'error', text: 'Failed to save API keys' });
    } finally {
      setSavingKeys(new Set());
    }
  };

  const handleDelete = async (keyName: string, keyLabel: string) => {
    if (!confirm(`Delete ${keyLabel}?`)) return;

    try {
      await deleteApiKey(keyName);
      setMessage({ type: 'success', text: 'API key deleted' });
      await loadKeys();
    } catch (err) {
      setMessage({ type: 'error', text: 'Failed to delete API key' });
    }
  };

  const handleKeyInputChange = (keyName: string, value: string) => {
    setKeyInputs(prev => ({ ...prev, [keyName]: value }));
  };

  // Cloud backup handlers
  const formatKeystoreError = (err: unknown): string => {
    const message = err instanceof Error ? err.message : 'Unknown error';
    // Check for keystore connection errors
    if (message.includes('keystore') || message.includes('connect') || message.includes('BadGateway')) {
      return 'Keystore server is unreachable. Please try again later.';
    }
    return message;
  };

  const handleUploadBackup = async () => {
    if (keys.length === 0) {
      setBackupMessage({ type: 'error', text: 'No API keys to backup' });
      return;
    }

    setIsUploading(true);
    setBackupMessage(null);

    try {
      const result = await backupKeysToCloud();
      setBackupMessage({ type: 'success', text: `Backed up ${result.key_count} keys to cloud` });
    } catch (err) {
      setBackupMessage({ type: 'error', text: formatKeystoreError(err) });
    } finally {
      setIsUploading(false);
    }
  };

  const handleDownloadBackup = async () => {
    setIsDownloading(true);
    setBackupMessage(null);

    try {
      const result = await restoreKeysFromCloud();
      await loadKeys();
      setBackupMessage({ type: 'success', text: `Restored ${result.key_count} keys from backup` });
    } catch (err) {
      setBackupMessage({ type: 'error', text: formatKeystoreError(err) });
    } finally {
      setIsDownloading(false);
    }
  };

  const handlePreviewCloudKeys = async () => {
    setIsPreviewing(true);
    setBackupMessage(null);

    try {
      const result = await previewKeysFromCloud();
      setPreviewKeys(result.keys);
      setPreviewModalOpen(true);
    } catch (err) {
      setBackupMessage({ type: 'error', text: formatKeystoreError(err) });
    } finally {
      setIsPreviewing(false);
    }
  };

  if (isLoading) {
    return (
      <div className="p-8 flex items-center justify-center">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 border-2 border-stark-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-slate-400">Loading API keys...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white mb-2">API Keys</h1>
        <p className="text-slate-400">
          Manage API keys for external services like web search and GitHub.
        </p>
      </div>

      {message && (
        <div
          className={`mb-6 px-4 py-3 rounded-lg ${
            message.type === 'success'
              ? 'bg-green-500/20 border border-green-500/50 text-green-400'
              : 'bg-red-500/20 border border-red-500/50 text-red-400'
          }`}
        >
          {message.text}
        </div>
      )}

      {/* Configuration Summary and Cloud Backup Row */}
      <div className="mb-6 flex flex-col lg:flex-row gap-6">
        {/* Installed Keys */}
        <div className="flex-1">
          <Card className="h-full">
            <CardHeader>
              <CardTitle>Installed Keys</CardTitle>
            </CardHeader>
            <CardContent>
              {keys.length === 0 ? (
                <div className="text-center py-8 text-slate-500">
                  <Key className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No API keys configured yet.</p>
                  <p className="text-sm mt-1">Add keys below to get started.</p>
                </div>
              ) : (
                <div className="flex flex-wrap gap-4">
                  {serviceConfigs.map((config) => {
                    const configuredKeys = config.keys.filter(k => isKeyConfigured(k.name));
                    if (configuredKeys.length === 0) return null;

                    return (
                      <div key={config.group} className="p-4 bg-slate-900/50 rounded-lg border border-slate-700 flex-1 min-w-[250px]">
                        <div className="flex items-center justify-between mb-2">
                          <p className="font-medium text-white">{config.label}</p>
                          {isGroupComplete(config) ? (
                            <span className="flex items-center gap-1 text-xs text-green-400">
                              <Check className="w-3 h-3" />
                              Complete
                            </span>
                          ) : (
                            <span className="flex items-center gap-1 text-xs text-yellow-400">
                              <X className="w-3 h-3" />
                              Partial
                            </span>
                          )}
                        </div>
                        <div className="text-sm text-slate-400 space-y-1">
                          {config.keys.map((keyConfig) => {
                            const configuredKey = getConfiguredKey(keyConfig.name);
                            return (
                              <p key={keyConfig.name} className="flex items-center gap-2">
                                {configuredKey ? (
                                  <Check className="w-3 h-3 text-green-400" />
                                ) : (
                                  <X className="w-3 h-3 text-slate-600" />
                                )}
                                <span className={configuredKey ? 'text-slate-300' : 'text-slate-600'}>
                                  {keyConfig.label}
                                </span>
                                {configuredKey && (
                                  <span className="font-mono text-slate-500">
                                    {configuredKey.key_preview}
                                  </span>
                                )}
                              </p>
                            );
                          })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Encrypted Cloud Backup */}
        <div className="lg:w-80 lg:flex-shrink-0">
          <Card className="h-full border-stark-500/30">
            <CardHeader>
              <div className="flex items-center gap-2">
                <Cloud className="w-5 h-5 text-stark-400" />
                <CardTitle>Encrypted Cloud Backup</CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex items-start gap-2 mb-4 p-2 bg-stark-500/10 rounded-lg">
                <Shield className="w-4 h-4 text-stark-400 mt-0.5 flex-shrink-0" />
                <p className="text-xs text-slate-400">
                  Encrypted with your burner wallet key. Only this instance can decrypt the backup.
                </p>
              </div>

              {backupMessage && (
                <div
                  className={`mb-4 px-3 py-2 rounded-lg text-sm flex items-start gap-2 ${
                    backupMessage.type === 'success'
                      ? 'bg-green-500/20 border border-green-500/50 text-green-400'
                      : 'bg-red-500/20 border border-red-500/50 text-red-400'
                  }`}
                >
                  {backupMessage.type === 'success' ? (
                    <CheckCircle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                  ) : (
                    <AlertCircle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                  )}
                  <span>{backupMessage.text}</span>
                </div>
              )}

              <div className="space-y-3">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={handleUploadBackup}
                  isLoading={isUploading}
                  disabled={keys.length === 0}
                  className="w-full"
                >
                  <Upload className="w-4 h-4 mr-2" />
                  Backup to Cloud
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={handlePreviewCloudKeys}
                  isLoading={isPreviewing}
                  className="w-full"
                >
                  <Eye className="w-4 h-4 mr-2" />
                  View Cloud Keys
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={handleDownloadBackup}
                  isLoading={isDownloading}
                  className="w-full"
                >
                  <Download className="w-4 h-4 mr-2" />
                  Restore from Cloud
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Service Groups and Info */}
      <div className="max-w-2xl space-y-6">
        {/* Service Groups */}
        {serviceConfigs.map((config) => (
            <Card key={config.group}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <CardTitle>{config.label}</CardTitle>
                    {isGroupComplete(config) && (
                      <span className="flex items-center gap-1 text-xs text-green-400 bg-green-500/20 px-2 py-1 rounded">
                        <Check className="w-3 h-3" />
                        Configured
                      </span>
                    )}
                    {isGroupPartial(config) && (
                      <span className="flex items-center gap-1 text-xs text-yellow-400 bg-yellow-500/20 px-2 py-1 rounded">
                        Partial
                      </span>
                    )}
                  </div>
                  <a
                    href={config.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-stark-400 hover:text-stark-300 inline-flex items-center gap-1 text-sm"
                  >
                    Get Keys
                    <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
                <p className="text-sm text-slate-400 mt-1">{config.description}</p>
              </CardHeader>
              <CardContent>
                <form onSubmit={(e) => handleSaveGroup(config, e)} className="space-y-4">
                  {config.keys.map((keyConfig) => {
                    const configuredKey = getConfiguredKey(keyConfig.name);
                    const isConfigured = !!configuredKey;

                    return (
                      <div key={keyConfig.name} className="space-y-2">
                        <div className="flex items-center justify-between">
                          <label className="block text-sm font-medium text-slate-300">
                            {keyConfig.label}
                          </label>
                          {isConfigured && (
                            <div className="flex items-center gap-2">
                              <span className="text-xs text-slate-500 font-mono">
                                {configuredKey.key_preview}
                              </span>
                              <button
                                type="button"
                                onClick={() => handleDelete(keyConfig.name, keyConfig.label)}
                                className="text-red-400 hover:text-red-300 p-1"
                                title="Delete this key"
                              >
                                <Trash2 className="w-3 h-3" />
                              </button>
                            </div>
                          )}
                        </div>
                        <div className="flex gap-2">
                          <Input
                            type={keyConfig.secret ? 'password' : 'text'}
                            value={keyInputs[keyConfig.name] || ''}
                            onChange={(e) => handleKeyInputChange(keyConfig.name, e.target.value)}
                            placeholder={isConfigured ? 'Enter new value to update' : `Enter ${keyConfig.label.toLowerCase()}`}
                            className="flex-1"
                          />
                          <Button
                            type="button"
                            variant="secondary"
                            size="sm"
                            onClick={() => handleSaveKey(keyConfig.name)}
                            isLoading={savingKeys.has(keyConfig.name)}
                            disabled={!keyInputs[keyConfig.name]?.trim()}
                          >
                            Save
                          </Button>
                        </div>
                      </div>
                    );
                  })}

                  {config.keys.length > 1 && (
                    <div className="pt-2 border-t border-slate-700">
                      <Button
                        type="submit"
                        isLoading={config.keys.some(k => savingKeys.has(k.name))}
                        disabled={!config.keys.some(k => keyInputs[k.name]?.trim())}
                      >
                        <Plus className="w-4 h-4 mr-2" />
                        Save All {config.label} Keys
                      </Button>
                    </div>
                  )}
                </form>
              </CardContent>
            </Card>
          ))}

          {/* Service Info */}
          <Card className="border-stark-500/30 bg-stark-500/5">
            <CardContent className="pt-6">
              <div className="flex items-start gap-4">
                <Key className="w-6 h-6 text-stark-400 flex-shrink-0" />
                <div>
                  <h4 className="font-medium text-white mb-3">Where to get API keys</h4>
                  <ul className="space-y-2 text-sm text-slate-400">
                    {serviceConfigs.map((service) => (
                      <li key={service.group} className="flex items-center gap-2">
                        <span className="text-slate-300 font-medium">{service.label}:</span>
                        <span>{service.description}</span>
                        <a
                          href={service.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-stark-400 hover:text-stark-300 inline-flex items-center gap-1"
                        >
                          <ExternalLink className="w-3 h-3" />
                        </a>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

      {/* Cloud Keys Preview Modal */}
      {previewModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          {/* Backdrop */}
          <div
            className="absolute inset-0 bg-black/70 backdrop-blur-sm"
            onClick={() => setPreviewModalOpen(false)}
          />
          {/* Modal */}
          <div className="relative bg-slate-800 border border-slate-700 rounded-xl shadow-2xl w-full max-w-lg max-h-[80vh] overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-stark-500/20 rounded-lg">
                  <Cloud className="w-5 h-5 text-stark-400" />
                </div>
                <div>
                  <h2 className="text-lg font-semibold text-white">Cloud Keys Preview</h2>
                  <p className="text-sm text-slate-400">{previewKeys.length} keys stored in cloud</p>
                </div>
              </div>
              <button
                onClick={() => setPreviewModalOpen(false)}
                className="text-slate-400 hover:text-white p-1"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            {/* Content */}
            <div className="p-6 overflow-y-auto max-h-[50vh]">
              {previewKeys.length === 0 ? (
                <div className="text-center py-8 text-slate-500">
                  <Cloud className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>No keys found in cloud backup.</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {previewKeys.map((key) => (
                    <div
                      key={key.key_name}
                      className="flex items-center justify-between p-3 bg-slate-900/50 rounded-lg border border-slate-700"
                    >
                      <div className="flex items-center gap-3">
                        <Key className="w-4 h-4 text-slate-500" />
                        <span className="text-sm font-medium text-white">{key.key_name}</span>
                      </div>
                      <span className="text-sm font-mono text-slate-400">{key.key_preview}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
            {/* Footer */}
            <div className="flex items-center justify-between px-6 py-4 border-t border-slate-700 bg-slate-900/30">
              <p className="text-xs text-slate-500">
                Use "Restore from Cloud" to apply these keys
              </p>
              <div className="flex gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setPreviewModalOpen(false)}
                >
                  Close
                </Button>
                <Button
                  size="sm"
                  onClick={() => {
                    setPreviewModalOpen(false);
                    handleDownloadBackup();
                  }}
                >
                  <Download className="w-4 h-4 mr-2" />
                  Restore Now
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
