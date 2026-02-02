import { useState, useEffect, useRef } from 'react';
import { Zap, Upload, Trash2, ExternalLink } from 'lucide-react';
import Card, { CardContent } from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { getSkills, uploadSkill, deleteSkill, setSkillEnabled, SkillInfo } from '@/lib/api';

export default function Skills() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    try {
      const data = await getSkills();
      setSkills(data);
    } catch (err) {
      setError('Failed to load skills');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setIsUploading(true);
    setError(null);

    try {
      await uploadSkill(file);
      await loadSkills();
    } catch (err) {
      setError('Failed to upload skill');
    } finally {
      setIsUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Are you sure you want to delete the skill "${name}"?`)) return;

    try {
      await deleteSkill(name);
      setSkills((prev) => prev.filter((s) => s.name !== name));
    } catch (err) {
      setError('Failed to delete skill');
    }
  };

  const handleToggleEnabled = async (name: string, currentEnabled: boolean) => {
    try {
      await setSkillEnabled(name, !currentEnabled);
      setSkills((prev) =>
        prev.map((s) => (s.name === name ? { ...s, enabled: !currentEnabled } : s))
      );
    } catch (err) {
      setError('Failed to update skill');
    }
  };

  if (isLoading) {
    return (
      <div className="p-4 sm:p-8 flex items-center justify-center">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 border-2 border-stark-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-slate-400">Loading skills...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 sm:p-8">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6 sm:mb-8">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold text-white mb-1 sm:mb-2">Skills</h1>
          <p className="text-sm sm:text-base text-slate-400">Extend your agent with custom skills</p>
        </div>
        <div>
          <input
            ref={fileInputRef}
            type="file"
            accept=".zip,.md"
            onChange={handleUpload}
            className="hidden"
          />
          <Button
            onClick={() => fileInputRef.current?.click()}
            isLoading={isUploading}
            className="w-full sm:w-auto"
          >
            <Upload className="w-4 h-4 mr-2" />
            Upload Skill
          </Button>
        </div>
      </div>

      {error && (
        <div className="mb-6 bg-red-500/20 border border-red-500/50 text-red-400 px-4 py-3 rounded-lg">
          {error}
        </div>
      )}

      {skills.length > 0 ? (
        <div className="grid gap-4">
          {skills.map((skill) => (
            <Card key={skill.name}>
              <CardContent>
                {/* Mobile: stacked layout, Desktop: side by side */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
                  {/* Main content */}
                  <div className="flex items-start sm:items-center gap-2 sm:gap-4 min-w-0">
                    {/* Icon - smaller on mobile */}
                    <div className="p-1.5 sm:p-3 bg-amber-500/20 rounded-lg shrink-0">
                      <Zap className="w-4 h-4 sm:w-6 sm:h-6 text-amber-400" />
                    </div>
                    <div className="min-w-0 flex-1">
                      {/* Title row */}
                      <div className="flex items-center gap-2 flex-wrap">
                        <h3 className="font-semibold text-white text-sm sm:text-base">{skill.name}</h3>
                        {skill.version && (
                          <span className="text-xs px-1.5 py-0.5 bg-slate-700 text-slate-400 rounded">
                            v{skill.version}
                          </span>
                        )}
                        {skill.homepage && (
                          <a
                            href={skill.homepage}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-slate-400 hover:text-stark-400"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <ExternalLink className="w-3.5 h-3.5 sm:w-4 sm:h-4" />
                          </a>
                        )}
                      </div>
                      {/* Source badge on separate line on mobile */}
                      {skill.source && (
                        <span className="inline-block text-xs px-1.5 py-0.5 bg-slate-700/50 text-slate-500 rounded mt-1">
                          {skill.source}
                        </span>
                      )}
                      {/* Description */}
                      {skill.description && (
                        <p className="text-xs sm:text-sm text-slate-400 mt-1.5">{skill.description}</p>
                      )}
                      {/* Tags */}
                      {skill.tags && skill.tags.length > 0 && (
                        <div className="flex flex-wrap gap-1 mt-2">
                          {skill.tags.map((tag) => (
                            <span
                              key={tag}
                              className="text-xs px-1.5 py-0.5 bg-stark-500/10 text-stark-400 rounded"
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                  {/* Action buttons - bottom right on mobile */}
                  <div className="flex items-center gap-2 self-end sm:self-center shrink-0">
                    <button
                      onClick={() => handleToggleEnabled(skill.name, skill.enabled)}
                      className={`px-2 py-1 text-xs rounded cursor-pointer transition-colors ${
                        skill.enabled
                          ? 'bg-green-500/20 text-green-400 hover:bg-green-500/30'
                          : 'bg-slate-700 text-slate-400 hover:bg-slate-600'
                      }`}
                    >
                      {skill.enabled ? 'Enabled' : 'Disabled'}
                    </button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDelete(skill.name)}
                      className="text-red-400 hover:text-red-300 hover:bg-red-500/20 p-1.5 sm:p-2"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="text-center py-12">
            <Zap className="w-12 h-12 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400 mb-4">No skills installed</p>
            <Button
              variant="secondary"
              onClick={() => fileInputRef.current?.click()}
            >
              <Upload className="w-4 h-4 mr-2" />
              Upload Your First Skill
            </Button>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
