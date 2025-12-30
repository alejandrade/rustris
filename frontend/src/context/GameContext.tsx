import { createContext, useContext, useState, useEffect, useCallback, useMemo, ReactNode } from "react";
import { gameService, Game } from "../services/GameService";
import { wineService, WineVersionInfo } from "../services/WineService";

export type { Game, WineVersionInfo };

interface GameContextType {
  games: Game[];
  selectedGame: Game | null;
  loading: boolean;
  availableWineVersions: WineVersionInfo[];
  setSelectedGame: (game: Game | null) => void;
  refreshGames: () => Promise<void>;
  refreshWineVersions: () => Promise<void>;
  updateGame: (game: Game) => void;
}

const GameContext = createContext<GameContextType | undefined>(undefined);

export function GameProvider({ children }: { children: ReactNode }) {
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGame, setSelectedGame] = useState<Game | null>(null);
  const [loading, setLoading] = useState(true);
  const [availableWineVersions, setAvailableWineVersions] = useState<WineVersionInfo[]>([]);

  const loadGames = async () => {
    try {
      const fetchedGames = await gameService.getGames();
      console.log("📥 Received games from backend:", fetchedGames.length);

      setGames(fetchedGames);
      if (fetchedGames.length > 0 && !selectedGame) {
        setSelectedGame(fetchedGames[0]);
      }

      setLoading(false);
    } catch (error) {
      console.error("Failed to load games:", error);
      setLoading(false);
    }
  };

  const refreshGames = useCallback(async () => {
    const fetchedGames = await gameService.getGames();
    setGames(fetchedGames);
    // Update selected game if it was updated
    if (selectedGame) {
      const updated = fetchedGames.find((g) => g.slug === selectedGame.slug);
      if (updated) {
        setSelectedGame(updated);
      }
    }
  }, [selectedGame]);

  const updateGame = useCallback((updatedGame: Game) => {
    setGames((prevGames) =>
      prevGames.map((g) => (g.slug === updatedGame.slug ? updatedGame : g))
    );
    if (selectedGame?.slug === updatedGame.slug) {
      setSelectedGame(updatedGame);
    }
  }, [selectedGame]);

  const refreshWineVersions = useCallback(async () => {
    try {
      const versions = await wineService.getAvailableVersions();
      setAvailableWineVersions(versions);
      console.log("🔄 Refreshed wine versions:", versions.length);
    } catch (error) {
      console.error("Failed to refresh wine versions:", error);
    }
  }, []);

  useEffect(() => {
    loadGames();
    refreshWineVersions();
  }, [refreshWineVersions]);

  const contextValue = useMemo(
    () => ({
      games,
      selectedGame,
      loading,
      availableWineVersions,
      setSelectedGame,
      refreshGames,
      refreshWineVersions,
      updateGame,
    }),
    [games, selectedGame, loading, availableWineVersions, refreshGames, refreshWineVersions, updateGame]
  );

  return (
    <GameContext.Provider value={contextValue}>
      {children}
    </GameContext.Provider>
  );
}

export function useGames() {
  const context = useContext(GameContext);
  if (context === undefined) {
    throw new Error("useGames must be used within a GameProvider");
  }
  return context;
}