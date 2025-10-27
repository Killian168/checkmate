import { create } from "zustand";
import { persist } from "zustand/middleware";

const Chess = require("chess.js");

interface AuthState {
  user: any | null; // Cognito user object
  isLoading: boolean;
  setUser: (user: any) => void;
  clearUser: () => void;
  setLoading: (loading: boolean) => void;
}

interface GameState {
  gameId: string | null;
  opponent: string | null;
  chess: any | null; // Chess instance
  isMyTurn: boolean;
  setGame: (gameId: string, opponent: string, fen?: string) => void;
  makeMove: (move: any) => void;
  endGame: () => void;
}

interface Store extends AuthState, GameState {}

export const useStore = create<Store>()(
  persist(
    (set, get) => ({
      // Auth state
      user: null,
      isLoading: false,
      setUser: (user) => set({ user }),
      clearUser: () => set({ user: null }),
      setLoading: (loading) => set({ isLoading: loading }),

      // Game state
      gameId: null,
      opponent: null,
      chess: null,
      isMyTurn: false,
      setGame: (
        gameId,
        opponent,
        fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      ) => {
        const chessInstance = new Chess(fen);
        set({ gameId, opponent, chess: chessInstance, isMyTurn: true });
      },
      makeMove: (move) => {
        const chess = get().chess;
        if (chess && chess.move(move)) {
          set({ isMyTurn: !get().isMyTurn });
        }
      },
      endGame: () =>
        set({ gameId: null, opponent: null, chess: null, isMyTurn: false }),
    }),
    {
      name: "chess-store",
      partialize: (state) => ({ user: state.user }), // Persist only user data
    },
  ),
);
