import { useState, useCallback, useRef } from 'react';
import {
  CanvasTemplate,
  CanvasElement,
} from '@/types/autoEdit';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Plus,
  Trash2,
  Type,
  Image as ImageIcon,
  Palette,
  Upload,
  AlertCircle,
  Save,
  FolderOpen,
} from 'lucide-react';
import { useAutoEdit } from '@/hooks/useAutoEdit';
import { TemplateLibrary } from './TemplateLibrary';
import { useTranslation } from 'react-i18next';

interface CanvasEditorProps {
  template: CanvasTemplate | null;
  onTemplateChange: (template: CanvasTemplate) => void;
}

export function CanvasEditor({ template, onTemplateChange }: CanvasEditorProps) {
  const { t } = useTranslation();
  const { saveCanvasTemplate } = useAutoEdit();
  const [selectedElementIndex, setSelectedElementIndex] = useState<number | null>(null);
  const [showLibrary, setShowLibrary] = useState(false);
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [saveTemplateName, setSaveTemplateName] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  // Initialize template if null
  const currentTemplate: CanvasTemplate = template || {
    id: `template_${Date.now()}`,
    name: 'New Template',
    background: { type: 'Color', value: '#000000' },
    elements: [],
  };

  const updateTemplate = useCallback((updates: Partial<CanvasTemplate>) => {
    onTemplateChange({ ...currentTemplate, ...updates });
  }, [currentTemplate, onTemplateChange]);

  // Background management
  const setBackgroundColor = useCallback((color: string) => {
    updateTemplate({
      background: { type: 'Color', value: color },
    });
  }, [updateTemplate]);

  const setBackgroundGradient = useCallback((color1: string, color2: string) => {
    updateTemplate({
      background: { type: 'Gradient', value: `${color1}:${color2}` },
    });
  }, [updateTemplate]);

  const setBackgroundImage = useCallback((path: string) => {
    updateTemplate({
      background: { type: 'Image', path },
    });
  }, [updateTemplate]);

  // Element management
  const addTextElement = useCallback(() => {
    const newElement: CanvasElement = {
      type: 'Text',
      content: 'New Text',
      font: 'Arial',
      size: 48,
      color: '#FFFFFF',
      position: { x: 50, y: 50 },
    };

    updateTemplate({
      elements: [...currentTemplate.elements, newElement],
    });
    setSelectedElementIndex(currentTemplate.elements.length);
  }, [currentTemplate, updateTemplate]);

  const addImageElement = useCallback((path: string) => {
    const newElement: CanvasElement = {
      type: 'Image',
      path,
      width: 200,
      height: 200,
      position: { x: 50, y: 50 },
    };

    updateTemplate({
      elements: [...currentTemplate.elements, newElement],
    });
    setSelectedElementIndex(currentTemplate.elements.length);
  }, [currentTemplate, updateTemplate]);

  const updateElement = useCallback((index: number, updates: Partial<CanvasElement>) => {
    const newElements = [...currentTemplate.elements];
    newElements[index] = { ...newElements[index], ...updates } as CanvasElement;

    updateTemplate({ elements: newElements });
  }, [currentTemplate, updateTemplate]);

  const deleteElement = useCallback((index: number) => {
    const newElements = currentTemplate.elements.filter((_, i) => i !== index);
    updateTemplate({ elements: newElements });

    if (selectedElementIndex === index) {
      setSelectedElementIndex(null);
    }
  }, [currentTemplate, selectedElementIndex, updateTemplate]);

  // Canvas click handling for element positioning
  const handleCanvasClick = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    if (selectedElementIndex === null) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 100;
    const y = ((e.clientY - rect.top) / rect.height) * 100;

    updateElement(selectedElementIndex, {
      position: { x: Math.max(0, Math.min(100, x)), y: Math.max(0, Math.min(100, y)) },
    } as Partial<CanvasElement>);
  }, [selectedElementIndex, updateElement]);

  const selectedElement = selectedElementIndex !== null
    ? currentTemplate.elements[selectedElementIndex]
    : null;

  // Template save/load handlers
  const handleSaveTemplate = useCallback(async () => {
    setSaveError(null);
    setIsSaving(true);

    try {
      const templateToSave: CanvasTemplate = {
        ...currentTemplate,
        name: saveTemplateName || currentTemplate.name,
      };

      await saveCanvasTemplate(templateToSave);
      setShowSaveDialog(false);
      setSaveTemplateName('');
    } catch (err) {
      console.error('Failed to save template:', err);
      setSaveError(err as string);
    } finally {
      setIsSaving(false);
    }
  }, [currentTemplate, saveTemplateName, saveCanvasTemplate]);

  const handleLoadTemplate = useCallback((loadedTemplate: CanvasTemplate) => {
    onTemplateChange(loadedTemplate);
  }, [onTemplateChange]);

  const openSaveDialog = useCallback(() => {
    setSaveTemplateName(currentTemplate.name);
    setSaveError(null);
    setShowSaveDialog(true);
  }, [currentTemplate.name]);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="p-4 border-b">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-semibold text-lg">{t('autoEdit.canvasEditor')}</h3>
            <p className="text-sm text-muted-foreground">
              {t('autoEdit.canvasEditorDescription')}
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowLibrary(true)}
            >
              <FolderOpen className="w-4 h-4 mr-2" />
              {t('autoEdit.loadTemplate')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={openSaveDialog}
            >
              <Save className="w-4 h-4 mr-2" />
              {t('autoEdit.saveTemplate')}
            </Button>
          </div>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Canvas Preview (9:16 aspect ratio) */}
        <div className="flex-1 p-6 flex items-center justify-center bg-muted/20">
          <div className="relative">
            <div
              ref={canvasRef}
              className="relative bg-black rounded-lg overflow-hidden shadow-2xl cursor-crosshair"
              style={{
                width: '360px',
                height: '640px',
                aspectRatio: '9/16',
              }}
              onClick={handleCanvasClick}
            >
              {/* Background layer */}
              <div
                className="absolute inset-0"
                style={{
                  background: currentTemplate.background.type === 'Color'
                    ? currentTemplate.background.value
                    : currentTemplate.background.type === 'Gradient'
                    ? `linear-gradient(${currentTemplate.background.value.split(':').join(', ')})`
                    : undefined,
                  backgroundImage: currentTemplate.background.type === 'Image'
                    ? `url(${currentTemplate.background.path})`
                    : undefined,
                  backgroundSize: 'cover',
                  backgroundPosition: 'center',
                }}
              />

              {/* Elements layer */}
              {currentTemplate.elements.map((element, index) => (
                <div
                  key={index}
                  className={`absolute cursor-pointer transition-all ${
                    selectedElementIndex === index ? 'ring-2 ring-primary' : ''
                  }`}
                  style={{
                    left: `${element.position.x}%`,
                    top: `${element.position.y}%`,
                    transform: 'translate(-50%, -50%)',
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedElementIndex(index);
                  }}
                >
                  {element.type === 'Text' ? (
                    <div
                      style={{
                        fontFamily: element.font,
                        fontSize: `${element.size}px`,
                        color: element.color,
                        textShadow: element.outline
                          ? `0 0 4px ${element.outline}, 0 0 8px ${element.outline}`
                          : undefined,
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {element.content}
                    </div>
                  ) : (
                    <img
                      src={element.path}
                      alt="Canvas element"
                      style={{
                        width: `${element.width}px`,
                        height: `${element.height}px`,
                        objectFit: 'contain',
                      }}
                    />
                  )}
                </div>
              ))}

              {/* Hint overlay */}
              {currentTemplate.elements.length === 0 && (
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="text-center text-white/50 p-4">
                    <p className="text-sm">Add text or images to get started</p>
                    <p className="text-xs mt-1">Click to position selected elements</p>
                  </div>
                </div>
              )}
            </div>

            {/* Dimensions label */}
            <div className="text-center mt-2 text-xs text-muted-foreground">
              1080 × 1920 (9:16 YouTube Shorts)
            </div>
          </div>
        </div>

        {/* Controls Panel */}
        <div className="w-80 border-l bg-card overflow-y-auto">
          <Tabs defaultValue="background" className="h-full">
            <TabsList className="w-full justify-start rounded-none border-b">
              <TabsTrigger value="background" className="flex-1">
                <Palette className="w-4 h-4 mr-1" />
                Background
              </TabsTrigger>
              <TabsTrigger value="elements" className="flex-1">
                <Plus className="w-4 h-4 mr-1" />
                Elements
              </TabsTrigger>
            </TabsList>

            {/* Background Tab */}
            <TabsContent value="background" className="p-4 space-y-4">
              <div className="space-y-2">
                <Label>Background Type</Label>
                <Select
                  value={currentTemplate.background.type}
                  onValueChange={(value: 'Color' | 'Gradient' | 'Image') => {
                    if (value === 'Color') setBackgroundColor('#000000');
                    else if (value === 'Gradient') setBackgroundGradient('#000000', '#333333');
                  }}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Color">Solid Color</SelectItem>
                    <SelectItem value="Gradient">Gradient</SelectItem>
                    <SelectItem value="Image">Image</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {currentTemplate.background.type === 'Color' && (
                <div className="space-y-2">
                  <Label>Color</Label>
                  <Input
                    type="color"
                    value={currentTemplate.background.value}
                    onChange={(e) => setBackgroundColor(e.target.value)}
                  />
                </div>
              )}

              {currentTemplate.background.type === 'Gradient' && (
                <>
                  <div className="space-y-2">
                    <Label>Color 1</Label>
                    <Input
                      type="color"
                      value={
                        currentTemplate.background.type === 'Gradient'
                          ? currentTemplate.background.value.split(':')[0]
                          : '#000000'
                      }
                      onChange={(e) => {
                        if (currentTemplate.background.type === 'Gradient') {
                          const color2 = currentTemplate.background.value.split(':')[1];
                          setBackgroundGradient(e.target.value, color2);
                        }
                      }}
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Color 2</Label>
                    <Input
                      type="color"
                      value={
                        currentTemplate.background.type === 'Gradient'
                          ? currentTemplate.background.value.split(':')[1]
                          : '#000000'
                      }
                      onChange={(e) => {
                        if (currentTemplate.background.type === 'Gradient') {
                          const color1 = currentTemplate.background.value.split(':')[0];
                          setBackgroundGradient(color1, e.target.value);
                        }
                      }}
                    />
                  </div>
                </>
              )}

              {currentTemplate.background.type === 'Image' && (
                <div className="space-y-2">
                  <Label>Image Path</Label>
                  <div className="flex gap-2">
                    <Input
                      placeholder="/path/to/image.jpg"
                      value={currentTemplate.background.path || ''}
                      onChange={(e) => setBackgroundImage(e.target.value)}
                    />
                    <Button size="icon" variant="outline">
                      <Upload className="w-4 h-4" />
                    </Button>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Image will be blurred and scaled to fit
                  </p>
                </div>
              )}
            </TabsContent>

            {/* Elements Tab */}
            <TabsContent value="elements" className="p-4 space-y-4">
              <div className="space-y-2">
                <Button
                  onClick={addTextElement}
                  className="w-full"
                  variant="outline"
                >
                  <Type className="w-4 h-4 mr-2" />
                  Add Text Element
                </Button>
                <Button
                  onClick={() => {
                    const path = prompt('Enter image path:');
                    if (path) addImageElement(path);
                  }}
                  className="w-full"
                  variant="outline"
                >
                  <ImageIcon className="w-4 h-4 mr-2" />
                  Add Image Element
                </Button>
              </div>

              <Separator />

              {/* Element list */}
              <div className="space-y-2">
                <Label>Elements ({currentTemplate.elements.length})</Label>
                {currentTemplate.elements.length === 0 ? (
                  <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      No elements added yet
                    </AlertDescription>
                  </Alert>
                ) : (
                  <div className="space-y-2">
                    {currentTemplate.elements.map((element, index) => (
                      <Card
                        key={index}
                        className={`cursor-pointer transition-all ${
                          selectedElementIndex === index ? 'ring-2 ring-primary' : ''
                        }`}
                        onClick={() => setSelectedElementIndex(index)}
                      >
                        <CardContent className="p-3">
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                              {element.type === 'Text' ? (
                                <Type className="w-4 h-4" />
                              ) : (
                                <ImageIcon className="w-4 h-4" />
                              )}
                              <span className="text-sm truncate">
                                {element.type === 'Text'
                                  ? element.content
                                  : 'Image'}
                              </span>
                            </div>
                            <Button
                              size="icon"
                              variant="ghost"
                              onClick={(e) => {
                                e.stopPropagation();
                                deleteElement(index);
                              }}
                            >
                              <Trash2 className="w-4 h-4 text-destructive" />
                            </Button>
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                )}
              </div>

              {/* Selected element properties */}
              {selectedElement && (
                <>
                  <Separator />
                  <div className="space-y-3">
                    <Label>Element Properties</Label>

                    {selectedElement.type === 'Text' && (
                      <>
                        <div className="space-y-2">
                          <Label>Text Content</Label>
                          <Input
                            value={selectedElement.content}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                content: e.target.value,
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Font Size</Label>
                          <Input
                            type="number"
                            value={selectedElement.size}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                size: parseInt(e.target.value),
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Color</Label>
                          <Input
                            type="color"
                            value={selectedElement.color}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                color: e.target.value,
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Outline Color (optional)</Label>
                          <Input
                            type="color"
                            value={selectedElement.outline || '#000000'}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                outline: e.target.value,
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                      </>
                    )}

                    {selectedElement.type === 'Image' && (
                      <>
                        <div className="space-y-2">
                          <Label>Width (px)</Label>
                          <Input
                            type="number"
                            value={selectedElement.width}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                width: parseInt(e.target.value),
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label>Height (px)</Label>
                          <Input
                            type="number"
                            value={selectedElement.height}
                            onChange={(e) =>
                              updateElement(selectedElementIndex!, {
                                height: parseInt(e.target.value),
                              } as Partial<CanvasElement>)
                            }
                          />
                        </div>
                      </>
                    )}

                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-2">
                        <Label>X Position (%)</Label>
                        <Input
                          type="number"
                          min="0"
                          max="100"
                          value={Math.round(selectedElement.position.x)}
                          onChange={(e) =>
                            updateElement(selectedElementIndex!, {
                              position: {
                                ...selectedElement.position,
                                x: parseInt(e.target.value),
                              },
                            } as Partial<CanvasElement>)
                          }
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Y Position (%)</Label>
                        <Input
                          type="number"
                          min="0"
                          max="100"
                          value={Math.round(selectedElement.position.y)}
                          onChange={(e) =>
                            updateElement(selectedElementIndex!, {
                              position: {
                                ...selectedElement.position,
                                y: parseInt(e.target.value),
                              },
                            } as Partial<CanvasElement>)
                          }
                        />
                      </div>
                    </div>
                  </div>
                </>
              )}
            </TabsContent>
          </Tabs>
        </div>
      </div>

      {/* Template Library Dialog */}
      <TemplateLibrary
        open={showLibrary}
        onOpenChange={setShowLibrary}
        onTemplateSelect={handleLoadTemplate}
      />

      {/* Save Template Dialog */}
      <Dialog open={showSaveDialog} onOpenChange={setShowSaveDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('autoEdit.saveTemplate')}</DialogTitle>
            <DialogDescription>
              {t('autoEdit.saveTemplateDescription')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="template-name">{t('autoEdit.templateName')}</Label>
              <Input
                id="template-name"
                placeholder={t('autoEdit.templateNamePlaceholder')}
                value={saveTemplateName}
                onChange={(e) => setSaveTemplateName(e.target.value)}
              />
            </div>

            {saveError && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{saveError}</AlertDescription>
              </Alert>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowSaveDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleSaveTemplate}
              disabled={isSaving || !saveTemplateName.trim()}
            >
              {isSaving ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  {t('common.saving')}
                </>
              ) : (
                <>
                  <Save className="w-4 h-4 mr-2" />
                  {t('common.save')}
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
