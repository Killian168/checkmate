import React, { useState } from "react";
import { Chessboard } from "react-chessboard";

interface ChessBoardProps {
  chess: any;
  onMove: (move: any) => void;
  isMyTurn: boolean;
  isBoardFlipped: boolean;
}

export default function ChessBoard({
  chess,
  onMove,
  isMyTurn,
  isBoardFlipped,
}: ChessBoardProps) {
  const [rightClickedSquares, setRightClickedSquares] = useState<any>({});
  const [optionSquares, setOptionSquares] = useState<any>({});

  function makeAMove(move: any) {
    const gameCopy = { ...chess };
    const result = gameCopy.move(move);
    if (result) {
      onMove(result);
      return true;
    }
    return false;
  }

  function getMoveOptions(square: string) {
    const moves = chess.moves({ square, verbose: true });
    const newSquares: any = {};
    moves.map((move: any) => {
      newSquares[move.to] = {
        background:
          chess.get(move.to) &&
          chess.get(move.to).color !== chess.get(square).color
            ? "radial-gradient(circle, rgba(0,0,0,.1) 85%, transparent 85%)"
            : "radial-gradient(circle, rgba(0,0,0,.1) 25%, transparent 25%)",
        borderRadius: "50%",
      };
      return move;
    });
    newSquares[square] = {
      background: "rgba(255, 255, 0, 0.4)",
    };
    setOptionSquares(newSquares);
  }

  function onSquareClick(square: string) {
    setRightClickedSquares({});
    if (!isMyTurn) return;

    function resetFirstMove(square: string) {
      setOptionSquares({});
    }

    if (!optionSquares[square]) {
      resetFirstMove(square);
      getMoveOptions(square);
    } else {
      resetFirstMove(square);
      // Make the move
      const move = {
        from: Object.keys(optionSquares)[0], // Assuming first is from
        to: square,
        promotion: "q", // Default promotion to queen
      };
      makeAMove(move);
    }
  }

  function onPieceDrop(sourceSquare: string, targetSquare: string) {
    if (!isMyTurn) return false;

    const move = {
      from: sourceSquare,
      to: targetSquare,
      promotion: "q",
    };
    const moveMade = makeAMove(move);
    setOptionSquares({});
    return moveMade;
  }

  function onSquareRightClick(square: string) {
    const colour = "rgba(0, 0, 255, 0.4)";
    setRightClickedSquares({
      ...rightClickedSquares,
      [square]:
        rightClickedSquares[square] &&
        rightClickedSquares[square].backgroundColor === colour
          ? undefined
          : { backgroundColor: colour },
    });
  }

  return (
    <div style={{ width: "500px", margin: "0 auto" }}>
      <Chessboard // @ts-ignore
        position={chess.fen()}
        onPieceDrop={onPieceDrop}
        onSquareClick={onSquareClick}
        onSquareRightClick={onSquareRightClick}
        boardOrientation={isBoardFlipped ? "black" : "white"}
        customSquareStyles={{
          ...optionSquares,
          ...rightClickedSquares,
        }}
        isDraggablePiece={({
          piece,
          sourceSquare,
        }: {
          piece: string;
          sourceSquare: string;
        }) => isMyTurn}
        arePiecesDraggable={isMyTurn}
      />
    </div>
  );
}
