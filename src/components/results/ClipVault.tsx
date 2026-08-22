import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type RefObject,
} from "react";
import { useNavigate } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  Film,
  PanelLeftClose,
  PanelLeftOpen,
  Play,
  Sparkles,
} from "lucide-react";
import { storageApi } from "@/api/storage";
import { Button } from "@/components/ui/button";
import { SpinnerCenter } from "@/components/ui/spinner";
import { VideoPlayer } from "@/components/video/VideoPlayer";
import { clipLabel } from "@/lib/clipLabel";
import { createClipThumbnailQueue } from "@/lib/clipThumbnailQueue";
import { clipSeconds } from "@/lib/eventLabel";
import { formatDuration } from "@/lib/utils";
import { type PinnedClipGroup, useAutoEditStore } from "@/stores/autoEditStore";
import type {
  ClipMetadata,
  ClipVaultGameGroup,
  ClipVaultSort,
} from "@/types/storage";

const PAGE_SIZE = 6;
const thumbnailQueue = createClipThumbnailQueue(
  ({ gameId, clipFilePath }) =>
    storageApi.ensureClipThumbnail(gameId, clipFilePath),
  2,
);

const selectionKey = (gameId: string, path: string) => `${gameId}\u0000${path}`;

interface SelectedClip {
  gameId: string;
  path: string;
  duration: number;
}

interface ActiveClip extends SelectedClip {
  clip: ClipMetadata;
}

export interface ClipVaultProps {
  onSelectionChange?: (groups: PinnedClipGroup[]) => void;
  onCreateMontage?: (groups: PinnedClipGroup[]) => void;
}

function useVisibleThumbnail(
  gameId: string,
  clip: ClipMetadata,
  onGenerated: (path: string) => void,
) {
  const ref = useRef<HTMLDivElement>(null);
  const [path, setPath] = useState(clip.thumbnail_path ?? null);
  const [failed, setFailed] = useState(false);
  const [version, setVersion] = useState(0);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setPath(clip.thumbnail_path ?? null);
    setFailed(false);
    setVersion(0);
    setVisible(false);
  }, [clip.file_path, clip.thumbnail_path]);

  const ensure = useCallback(() => {
    const pending = thumbnailQueue.request({
      gameId,
      clipFilePath: clip.file_path,
    });
    if (!pending) return;
    void pending
      .then((generatedPath) => {
        setPath(generatedPath);
        setFailed(false);
        setVersion(Date.now());
        onGenerated(generatedPath);
      })
      .catch(() => {
        setFailed(true);
      });
  }, [clip.file_path, gameId, onGenerated]);

  useEffect(() => {
    const node = ref.current;
    if (!node || typeof IntersectionObserver === "undefined") {
      setVisible(true);
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "160px" },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [clip.file_path]);

  useEffect(() => {
    if (visible && !path) ensure();
  }, [ensure, path, visible]);

  const onImageError = useCallback(() => {
    setFailed(true);
    setPath(null);
    if (visible) ensure();
  }, [ensure, visible]);

  return {
    ref,
    src:
      path && !failed
        ? `${convertFileSrc(path)}${version ? `?v=${version}` : ""}`
        : null,
    onImageError,
  };
}

interface VaultThumbnailProps {
  gameId: string;
  clip: ClipMetadata;
  onGenerated: (path: string) => void;
}

function VaultThumbnail({ gameId, clip, onGenerated }: VaultThumbnailProps) {
  const { ref, src, onImageError } = useVisibleThumbnail(
    gameId,
    clip,
    onGenerated,
  );
  return (
    <div ref={ref as RefObject<HTMLDivElement>} className="absolute inset-0">
      {src ? (
        <img
          src={src}
          alt=""
          loading="lazy"
          className="h-full w-full object-contain"
          onError={onImageError}
        />
      ) : (
        <Film
          className="absolute inset-0 m-auto h-9 w-9 text-muted-foreground"
          aria-hidden="true"
        />
      )}
    </div>
  );
}

export function ClipVault({
  onSelectionChange,
  onCreateMontage,
}: ClipVaultProps) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const setPinnedClips = useAutoEditStore((state) => state.setPinnedClips);
  const setSelectedGameIds = useAutoEditStore(
    (state) => state.setSelectedGameIds,
  );
  const targetDuration = useAutoEditStore((state) => state.targetDuration);

  const [groups, setGroups] = useState<ClipVaultGameGroup[]>([]);
  const [sortOrder, setSortOrder] = useState<ClipVaultSort>("best");
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [skippedItems, setSkippedItems] = useState(0);
  const [selected, setSelected] = useState<Map<string, SelectedClip>>(
    () => new Map(),
  );
  const [activeClip, setActiveClip] = useState<ActiveClip | null>(null);
  const [eventListOpen, setEventListOpen] = useState(true);
  const [status, setStatus] = useState<"loading" | "ready" | "error">(
    "loading",
  );
  const [loadingMore, setLoadingMore] = useState(false);
  const requestId = useRef(0);

  const loadPage = useCallback(
    async (reset: boolean, cursor: string | null, sort: ClipVaultSort) => {
      const id = ++requestId.current;
      if (reset) setStatus("loading");
      else setLoadingMore(true);
      try {
        const page = await storageApi.listClipVaultPage({
          sort,
          cursor,
          game_limit: PAGE_SIZE,
        });
        if (id !== requestId.current) return;
        setGroups((current) =>
          reset ? page.groups : [...current, ...page.groups],
        );
        setNextCursor(page.next_cursor);
        // The storage page reports the library-wide corrupt-row count, so later
        // pages must not add the same omissions again.
        setSkippedItems((current) =>
          reset
            ? page.skipped_item_count
            : Math.max(current, page.skipped_item_count),
        );
        setStatus("ready");
      } catch {
        if (id !== requestId.current) return;
        if (reset) setStatus("error");
      } finally {
        if (id === requestId.current) setLoadingMore(false);
      }
    },
    [],
  );

  useEffect(() => {
    setGroups([]);
    setNextCursor(null);
    setSkippedItems(0);
    void loadPage(true, null, sortOrder);
  }, [loadPage, sortOrder]);

  useEffect(() => {
    if (groups.length === 0) return;
    const activeStillPresent =
      activeClip &&
      groups.some(
        (group) =>
          group.game_id === activeClip.gameId &&
          group.clips.some((clip) => clip.file_path === activeClip.path),
      );
    if (activeStillPresent) return;
    const firstGroup = groups.find((group) => group.clips.length > 0);
    const firstClip = firstGroup?.clips[0];
    if (firstGroup && firstClip) {
      setActiveClip({
        gameId: firstGroup.game_id,
        path: firstClip.file_path,
        duration: firstClip.duration,
        clip: firstClip,
      });
    }
  }, [activeClip, groups]);

  const selectedGroups = useMemo(() => {
    const byGame = new Map<string, string[]>();
    for (const item of selected.values()) {
      const paths = byGame.get(item.gameId) ?? [];
      if (!paths.includes(item.path)) paths.push(item.path);
      byGame.set(item.gameId, paths);
    }
    return [...byGame.entries()].map(([gameId, paths]) => ({ gameId, paths }));
  }, [selected]);

  const totalDuration = useMemo(
    () => [...selected.values()].reduce((sum, clip) => sum + clip.duration, 0),
    [selected],
  );

  useEffect(() => {
    onSelectionChange?.(selectedGroups);
  }, [onSelectionChange, selectedGroups]);

  const toggleSelection = useCallback((gameId: string, clip: ClipMetadata) => {
    setSelected((current) => {
      const next = new Map(current);
      const key = selectionKey(gameId, clip.file_path);
      if (next.has(key)) next.delete(key);
      else {
        next.set(key, {
          gameId,
          path: clip.file_path,
          duration: Math.max(0, clip.duration),
        });
      }
      return next;
    });
  }, []);

  const toggleGroup = useCallback((group: ClipVaultGameGroup) => {
    setSelected((current) => {
      const next = new Map(current);
      const allSelected = group.clips.every((clip) =>
        next.has(selectionKey(group.game_id, clip.file_path)),
      );
      for (const clip of group.clips) {
        const key = selectionKey(group.game_id, clip.file_path);
        if (allSelected) next.delete(key);
        else {
          next.set(key, {
            gameId: group.game_id,
            path: clip.file_path,
            duration: Math.max(0, clip.duration),
          });
        }
      }
      return next;
    });
  }, []);

  const updateThumbnail = useCallback(
    (gameId: string, filePath: string, thumbnailPath: string) => {
      setGroups((current) =>
        current.map((group) =>
          group.game_id !== gameId
            ? group
            : {
                ...group,
                clips: group.clips.map((clip) =>
                  clip.file_path === filePath
                    ? { ...clip, thumbnail_path: thumbnailPath }
                    : clip,
                ),
              },
        ),
      );
    },
    [],
  );

  const makeMontage = useCallback(() => {
    if (selectedGroups.length === 0) return;
    setPinnedClips({ groups: selectedGroups });
    setSelectedGameIds(selectedGroups.map((group) => group.gameId));
    onCreateMontage?.(selectedGroups);
    if (!onCreateMontage) void navigate({ to: "/auto-edit", search: {} });
  }, [
    navigate,
    onCreateMontage,
    selectedGroups,
    setPinnedClips,
    setSelectedGameIds,
  ]);

  if (status === "loading") {
    return (
      <div data-testid="clip-vault-loading">
        <SpinnerCenter label={t("results.clips.loading")} />
      </div>
    );
  }

  if (status === "error") {
    return (
      <div
        className="rounded-lg border border-destructive/40 bg-destructive/10 p-6 text-center"
        data-testid="clip-vault-error"
      >
        <p className="text-sm text-muted-foreground">
          {t("results.clips.loadError")}
        </p>
        <Button
          className="mt-4"
          variant="outline"
          onClick={() => void loadPage(true, null, sortOrder)}
        >
          {t("results.refresh")}
        </Button>
      </div>
    );
  }

  if (groups.length === 0) {
    return (
      <div
        className="flex flex-col items-center justify-center rounded-lg border border-dashed border-white/10 p-10 text-center"
        data-testid="clip-vault-empty"
      >
        <Film
          className="mb-3 h-10 w-10 text-muted-foreground"
          aria-hidden="true"
        />
        <h3 className="font-semibold">{t("results.clips.emptyTitle")}</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          {t("results.clips.emptyDescription")}
        </p>
      </div>
    );
  }

  return (
    <section
      aria-label={t("results.clips.title")}
      className={selected.size > 0 ? "pb-32" : undefined}
    >
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">{t("results.clips.title")}</h2>
          <p className="text-sm text-muted-foreground">
            {t("results.clips.description")}
          </p>
        </div>
        <div
          className="flex rounded-md border border-white/10 p-1"
          aria-label={t("results.clips.sortLabel")}
        >
          <Button
            size="sm"
            variant={sortOrder === "best" ? "secondary" : "ghost"}
            aria-pressed={sortOrder === "best"}
            onClick={() => setSortOrder("best")}
          >
            {t("results.clips.sortRecommended")}
          </Button>
          <Button
            size="sm"
            variant={sortOrder === "newest" ? "secondary" : "ghost"}
            aria-pressed={sortOrder === "newest"}
            onClick={() => setSortOrder("newest")}
          >
            {t("results.clips.sortNewest")}
          </Button>
        </div>
      </div>

      {skippedItems > 0 && (
        <p
          className="mb-4 flex items-center gap-2 text-sm text-amber-400"
          role="status"
          data-testid="clip-vault-skipped-warning"
        >
          <AlertTriangle className="h-4 w-4" aria-hidden="true" />
          {t("results.clips.skippedItems", { count: skippedItems })}
        </p>
      )}

      <div className="overflow-hidden rounded-xl border border-white/10 bg-black/10">
        <div className="flex min-h-[32rem] flex-col md:flex-row">
          <aside
            className={`${eventListOpen ? "block w-full md:w-80" : "hidden w-0 overflow-hidden border-r-0 md:block"} max-h-80 shrink-0 border-b border-white/10 bg-gaming-sidebar/40 transition-[width] duration-200 md:max-h-none md:border-b-0 md:border-r`}
            aria-label={t("results.clips.title")}
          >
            <div className="h-full overflow-y-auto p-3">
              {groups.map((group) => {
                const allSelected = group.clips.every((clip) =>
                  selected.has(selectionKey(group.game_id, clip.file_path)),
                );
                return (
                  <section
                    key={group.game_id}
                    data-testid={`clip-vault-game-${group.game_id}`}
                    className="mb-5 last:mb-0"
                  >
                    <header className="mb-2 flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <h3 className="truncate text-sm font-semibold">
                          {group.game?.champion ||
                            `${t("results.clips.game")} ${group.game_id}`}
                          {group.game?.result && (
                            <span className="ml-2 text-muted-foreground">
                              {t(
                                `game.result.${group.game.result.toLowerCase()}`,
                              )}
                            </span>
                          )}
                        </h3>
                        <p className="text-xs text-muted-foreground">
                          {group.game?.start_time
                            ? new Intl.DateTimeFormat(i18n.language, {
                                dateStyle: "medium",
                                timeStyle: "short",
                              }).format(new Date(group.game.start_time))
                            : t("results.clips.unknownDate")}
                        </p>
                      </div>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => toggleGroup(group)}
                        aria-pressed={allSelected}
                      >
                        {allSelected
                          ? t("results.clips.clearGameSelection")
                          : t("results.clips.selectGame")}
                      </Button>
                    </header>
                    <div
                      className="space-y-1"
                      data-testid={`clip-vault-grid-${group.game_id}`}
                    >
                      {group.clips.map((clip, index) => {
                        const { title, reasons } = clipLabel(clip);
                        const label = t(title.key, title.params);
                        const key = selectionKey(group.game_id, clip.file_path);
                        const isActive =
                          activeClip?.path === clip.file_path &&
                          activeClip.gameId === group.game_id;
                        return (
                          <article
                            key={clip.file_path}
                            data-testid={`clip-vault-card-${clip.file_path}`}
                            className={`group flex overflow-hidden rounded-lg border ${isActive ? "border-gaming-cyan/70 bg-gaming-cyan/10" : "border-transparent hover:border-white/10 hover:bg-white/5"}`}
                          >
                            <button
                              type="button"
                              className="relative h-16 w-28 shrink-0 bg-black text-left"
                              onClick={() =>
                                setActiveClip({
                                  gameId: group.game_id,
                                  path: clip.file_path,
                                  duration: clip.duration,
                                  clip,
                                })
                              }
                              aria-label={`${t("results.clips.play")}: ${label}`}
                            >
                              <VaultThumbnail
                                gameId={group.game_id}
                                clip={clip}
                                onGenerated={(path) =>
                                  updateThumbnail(
                                    group.game_id,
                                    clip.file_path,
                                    path,
                                  )
                                }
                              />
                              <span className="absolute inset-0 flex items-center justify-center bg-black/20">
                                <Play
                                  className="h-5 w-5 text-white"
                                  aria-hidden="true"
                                />
                              </span>
                            </button>
                            <div className="min-w-0 flex-1 p-2">
                              <div className="flex items-start gap-1">
                                <button
                                  type="button"
                                  className="min-w-0 flex-1 text-left"
                                  onClick={() =>
                                    setActiveClip({
                                      gameId: group.game_id,
                                      path: clip.file_path,
                                      duration: clip.duration,
                                      clip,
                                    })
                                  }
                                >
                                  <p className="truncate text-sm font-medium">
                                    {label}
                                  </p>
                                  <p className="truncate text-xs text-muted-foreground">
                                    {reasons.length > 0
                                      ? reasons
                                          .map((reason) =>
                                            t(reason.key, reason.params),
                                          )
                                          .join(" · ")
                                      : t("home.clips.seconds", {
                                          count: clipSeconds(clip.duration),
                                        })}
                                  </p>
                                </button>
                                <input
                                  type="checkbox"
                                  checked={selected.has(key)}
                                  onChange={() =>
                                    toggleSelection(group.game_id, clip)
                                  }
                                  aria-label={`${t("results.clips.select")}: ${label}`}
                                  className="mt-1"
                                />
                              </div>
                              {sortOrder === "best" && index < 3 && (
                                <span className="text-[10px] font-bold text-gaming-cyan">
                                  {t("results.clips.gameRank", {
                                    rank: index + 1,
                                  })}
                                </span>
                              )}
                            </div>
                          </article>
                        );
                      })}
                    </div>
                  </section>
                );
              })}
            </div>
          </aside>
          <div className="min-w-0 flex-1 p-3 md:p-5">
            <div className="mb-3 flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setEventListOpen((open) => !open)}
                aria-label={
                  eventListOpen ? t("common.close") : t("results.clips.title")
                }
                aria-expanded={eventListOpen}
                className="hidden md:inline-flex"
              >
                {eventListOpen ? (
                  <PanelLeftClose className="h-4 w-4" aria-hidden="true" />
                ) : (
                  <PanelLeftOpen className="h-4 w-4" aria-hidden="true" />
                )}
              </Button>
              <div className="min-w-0">
                <h3 className="truncate font-semibold">
                  {activeClip
                    ? t(
                        clipLabel(activeClip.clip).title.key,
                        clipLabel(activeClip.clip).title.params,
                      )
                    : t("results.clips.title")}
                </h3>
                {activeClip && (
                  <p className="text-sm text-muted-foreground">
                    {t("home.clips.seconds", {
                      count: clipSeconds(activeClip.duration),
                    })}
                  </p>
                )}
              </div>
            </div>
            {activeClip ? (
              <VideoPlayer
                src={convertFileSrc(activeClip.path)}
                title={t(
                  clipLabel(activeClip.clip).title.key,
                  clipLabel(activeClip.clip).title.params,
                )}
                className="h-[min(55vh,28rem)] min-h-72 w-full md:h-[min(68vh,48rem)]"
              />
            ) : (
              <div className="flex h-[min(55vh,28rem)] min-h-72 items-center justify-center rounded-lg bg-black text-sm text-muted-foreground md:h-[min(68vh,48rem)]">
                {t("results.clips.emptyDescription")}
              </div>
            )}
          </div>
        </div>
        <div className="border-t border-white/10 p-3 md:hidden">
          <p className="text-xs text-muted-foreground">
            {t("results.clips.clipCount", {
              count: groups.reduce(
                (total, group) => total + group.clips.length,
                0,
              ),
            })}
          </p>
        </div>
      </div>

      {nextCursor && (
        <div className="mt-8 text-center">
          <Button
            variant="outline"
            disabled={loadingMore}
            onClick={() => void loadPage(false, nextCursor, sortOrder)}
          >
            {loadingMore
              ? t("results.clips.loadingMore")
              : t("results.clips.loadMore")}
          </Button>
        </div>
      )}

      {selected.size > 0 && (
        <div
          className="fixed bottom-4 left-4 right-4 z-40 mx-auto max-w-5xl rounded-xl border border-primary/30 bg-background/95 p-4 shadow-2xl backdrop-blur"
          data-testid="clip-vault-action-bar"
          role="region"
          aria-label={t("results.clips.selectionActions")}
        >
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <p className="font-medium">
                {t("results.clips.selectionSummary", {
                  clips: selected.size,
                  games: selectedGroups.length,
                  duration: formatDuration(totalDuration),
                })}
              </p>
              {totalDuration > targetDuration && (
                <p className="mt-1 text-sm text-amber-400" role="alert">
                  {t("results.clips.overTargetWarning")}
                </p>
              )}
            </div>
            <div className="flex gap-2">
              <Button variant="ghost" onClick={() => setSelected(new Map())}>
                {t("results.clips.clearAll")}
              </Button>
              <Button onClick={makeMontage} data-testid="create-montage-button">
                <Sparkles className="mr-2 h-4 w-4" aria-hidden="true" />
                {t("results.clips.createMontage")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
