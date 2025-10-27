"use client";

import { useState, useEffect } from "react";
import { useStore } from "@/lib/store";
import { useWebSocket } from "@/hooks/useWebSocket";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import ChessBoard from "@/components/ChessBoard";

export default function PlayPage() {
  const {
    gameId,
    opponent,
    chess,
    isMyTurn,
    setGame,
    makeMove,
    endGame,
    user,
  } = useStore();
  const [isMatchmaking, setIsMatchmaking] = useState(false);
  const [statusMessage, setStatusMessage] = useState("");

  const wsUrl = process.env.NEXT_PUBLIC_WS_URL!;

  const handleWebSocketMessage = (data: any) => {
    if (data.type === "match_found") {
      setGame(data.gameId, data.opponent);
      setIsMatchmaking(false);
      setStatusMessage("Game started!");
    } else if (data.type === "move") {
      if (data.gameId === gameId) {
        makeMove(data.move);
        setStatusMessage(
          `Opponent moved: ${data.move.from} to ${data.move.to}`,
        );
      }
    } else if (data.type === "game_over") {
      if (data.gameId === gameId) {
        setStatusMessage(data.message);
        endGame();
      }
    }
  };

  const { isConnected, sendMessage } = useWebSocket({
    url: wsUrl,
    onMessage: handleWebSocketMessage,
    onConnect: () => setStatusMessage("Connected to server"),
    onDisconnect: () => setStatusMessage("Disconnected from server"),
  });

  const handleMatchmaking = () => {
    if (!user) {
      setStatusMessage("Please log in to play");
      return;
    }
    setIsMatchmaking(true);
    setStatusMessage("Looking for opponent...");
    sendMessage({ type: "find_match", userId: user.username });
  };

  const handleMove = (move: any) => {
    makeMove(move);
    sendMessage({ type: "move", gameId, move });
    setStatusMessage(`You moved: ${move.from} to ${move.to}`);
  };

  const handleEndGame = () => {
    sendMessage({ type: "end_game", gameId });
    endGame();
    setStatusMessage("Game ended");
  };

  useEffect(() => {
    if (!isConnected && isMatchmaking) {
      setStatusMessage("Lost connection, reconnecting...");
    }
  }, [isConnected, isMatchmaking]);

  return (
    <div className="flex flex-col items-center min-h-screen bg-gray-50 p-8">
      {!gameId ? (
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle>Play Chess</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p>Find an opponent and start playing!</p>
            <Button
              onClick={handleMatchmaking}
              disabled={isMatchmaking || !isConnected}
              className="w-full"
            >
              {isMatchmaking ? "Finding Opponent..." : "Play vs Human"}
            </Button>
            {statusMessage && (
              <p className="text-center text-sm">{statusMessage}</p>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          <div className="text-center">
            <p>Playing against: {opponent}</p>
            <p>Turn: {isMyTurn ? "Your turn" : "Opponent's turn"}</p>
            {statusMessage && <p className="text-sm">{statusMessage}</p>}
          </div>
          {chess && (
            <ChessBoard
              chess={chess}
              onMove={handleMove}
              isMyTurn={isMyTurn}
              isBoardFlipped={false} // Or determine based on player color
            />
          )}
          <Button onClick={handleEndGame} variant="destructive">
            End Game
          </Button>
        </div>
      )}
    </div>
  );
}
