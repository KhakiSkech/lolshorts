import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, Trash2, FileImage, Type, AlertCircle } from 'lucide-react';
import { CanvasTemplate, CanvasTemplateInfo } from '@/types/autoEdit';
import { useAutoEdit } from '@/hooks/useAutoEdit';

interface TemplateLibraryProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onTemplateSelect: (template: CanvasTemplate) => void;
}

export function TemplateLibrary({ open, onOpenChange, onTemplateSelect }: TemplateLibraryProps) {
  const { t } = useTranslation();
  const { listCanvasTemplates, loadCanvasTemplate, deleteCanvasTemplate } = useAutoEdit();

  const [templates, setTemplates] = useState<CanvasTemplateInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const fetchTemplates = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const templateList = await listCanvasTemplates();
      setTemplates(templateList);
    } catch (err) {
      console.error('Failed to fetch templates:', err);
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  }, [listCanvasTemplates]);

  useEffect(() => {
    if (open) {
      fetchTemplates();
    }
  }, [open, fetchTemplates]);

  const handleLoadTemplate = async (templateId: string) => {
    try {
      const template = await loadCanvasTemplate(templateId);
      onTemplateSelect(template);
      onOpenChange(false);
    } catch (err) {
      console.error('Failed to load template:', err);
      setError(err as string);
    }
  };

  const handleDeleteTemplate = async (templateId: string) => {
    setDeletingId(templateId);

    try {
      await deleteCanvasTemplate(templateId);
      await fetchTemplates(); // Refresh list
    } catch (err) {
      console.error('Failed to delete template:', err);
      setError(err as string);
    } finally {
      setDeletingId(null);
    }
  };

  const getBackgroundPreview = (template: CanvasTemplateInfo): string => {
    // Simple preview text based on template name
    return template.name;
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t('autoEdit.templateLibrary')}</DialogTitle>
          <DialogDescription>
            {t('autoEdit.templateLibraryDescription')}
          </DialogDescription>
        </DialogHeader>

        {error && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin" />
            <span className="ml-2">{t('common.loading')}</span>
          </div>
        ) : templates.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            <FileImage className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>{t('autoEdit.noTemplates')}</p>
            <p className="text-sm mt-2">{t('autoEdit.createTemplateHint')}</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {templates.map((template) => (
              <Card key={template.id} className="relative">
                <CardHeader>
                  <CardTitle className="text-sm font-medium flex items-center gap-2">
                    <FileImage className="w-4 h-4" />
                    {template.name}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="aspect-[9/16] bg-muted rounded-lg flex items-center justify-center text-muted-foreground text-xs">
                    {getBackgroundPreview(template)}
                  </div>
                  <div className="mt-2 flex items-center gap-2">
                    <Badge variant="secondary" className="text-xs">
                      <Type className="w-3 h-3 mr-1" />
                      {template.element_count} {t('autoEdit.elements')}
                    </Badge>
                  </div>
                </CardContent>
                <CardFooter className="flex gap-2">
                  <Button
                    variant="default"
                    size="sm"
                    className="flex-1"
                    onClick={() => handleLoadTemplate(template.id)}
                  >
                    {t('autoEdit.loadTemplate')}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDeleteTemplate(template.id)}
                    disabled={deletingId === template.id}
                  >
                    {deletingId === template.id ? (
                      <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                      <Trash2 className="w-4 h-4" />
                    )}
                  </Button>
                </CardFooter>
              </Card>
            ))}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
