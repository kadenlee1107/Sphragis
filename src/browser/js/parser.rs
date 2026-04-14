// Bat_OS — JavaScript Parser
// Converts token stream into an AST (Abstract Syntax Tree).
// Recursive descent parser with operator precedence climbing.

use super::lexer::{Token, TokenType, MAX_TOKENS};
use super::ast::*;

/// Parse tokens into an AST.
pub fn parse(tokens: &[Token], ast: &mut Ast) {
    ast.count = 0;
    let mut pos = 0usize;

    let program = ast.alloc().unwrap_or(0);
    ast.nodes[program as usize].kind = NodeKind::Program;
    ast.root = program;

    let mut last_stmt: u16 = 0xFFFF;

    while pos < tokens.len() && tokens[pos].token_type != TokenType::Eof {
        if let Some(stmt) = parse_statement(tokens, &mut pos, ast) {
            if last_stmt == 0xFFFF {
                ast.nodes[program as usize].left = stmt;
            } else {
                ast.nodes[last_stmt as usize].next = stmt;
            }
            last_stmt = stmt;
        } else {
            pos += 1; // skip unparseable token
        }
    }
}

fn parse_statement(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    if *pos >= tokens.len() { return None; }

    match tokens[*pos].token_type {
        TokenType::Var | TokenType::Let | TokenType::Const => parse_var_decl(tokens, pos, ast),
        TokenType::Function => parse_function_decl(tokens, pos, ast),
        TokenType::If => parse_if(tokens, pos, ast),
        TokenType::While => parse_while(tokens, pos, ast),
        TokenType::For => parse_for(tokens, pos, ast),
        TokenType::Return => parse_return(tokens, pos, ast),
        TokenType::LeftBrace => parse_block(tokens, pos, ast),
        TokenType::Break => { *pos += 1; skip_semi(tokens, pos); let n = ast.alloc()?; ast.nodes[n as usize].kind = NodeKind::BreakStatement; Some(n) }
        TokenType::Continue => { *pos += 1; skip_semi(tokens, pos); let n = ast.alloc()?; ast.nodes[n as usize].kind = NodeKind::ContinueStatement; Some(n) }
        _ => {
            let expr = parse_expression(tokens, pos, ast, 0)?;
            skip_semi(tokens, pos);
            let stmt = ast.alloc()?;
            ast.nodes[stmt as usize].kind = NodeKind::ExprStatement;
            ast.nodes[stmt as usize].left = expr;
            Some(stmt)
        }
    }
}

fn parse_var_decl(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip var/let/const
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::VarDecl;

    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
        ast.nodes[node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
        *pos += 1;
    }

    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Assign {
        *pos += 1;
        if let Some(init) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].left = init;
        }
    }
    skip_semi(tokens, pos);
    Some(node)
}

fn parse_function_decl(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'function'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::FunctionDecl;

    // Function name
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
        ast.nodes[node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
        *pos += 1;
    }

    // Parameters
    expect(tokens, pos, TokenType::LeftParen);
    let mut pcount = 0u8;
    while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
        if tokens[*pos].token_type == TokenType::Identifier {
            let param = ast.alloc()?;
            ast.nodes[param as usize].kind = NodeKind::Identifier;
            ast.nodes[param as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
            if (pcount as usize) < 8 {
                ast.nodes[node as usize].params[pcount as usize] = param;
                pcount += 1;
            }
            *pos += 1;
        }
        if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
    }
    ast.nodes[node as usize].param_count = pcount;
    expect(tokens, pos, TokenType::RightParen);

    // Body
    if let Some(body) = parse_block(tokens, pos, ast) {
        ast.nodes[node as usize].body = body;
    }

    Some(node)
}

fn parse_if(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'if'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::IfStatement;

    expect(tokens, pos, TokenType::LeftParen);
    if let Some(cond) = parse_expression(tokens, pos, ast, 0) {
        ast.nodes[node as usize].left = cond;
    }
    expect(tokens, pos, TokenType::RightParen);

    if let Some(then_body) = parse_statement(tokens, pos, ast) {
        ast.nodes[node as usize].right = then_body;
    }

    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Else {
        *pos += 1;
        if let Some(else_body) = parse_statement(tokens, pos, ast) {
            ast.nodes[node as usize].extra = else_body;
        }
    }

    Some(node)
}

fn parse_while(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1;
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::WhileStatement;

    expect(tokens, pos, TokenType::LeftParen);
    if let Some(cond) = parse_expression(tokens, pos, ast, 0) {
        ast.nodes[node as usize].left = cond;
    }
    expect(tokens, pos, TokenType::RightParen);

    if let Some(body) = parse_statement(tokens, pos, ast) {
        ast.nodes[node as usize].right = body;
    }
    Some(node)
}

fn parse_for(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1;
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ForStatement;

    expect(tokens, pos, TokenType::LeftParen);
    // Init
    if *pos < tokens.len() && tokens[*pos].token_type != TokenType::Semicolon {
        if let Some(init) = parse_statement(tokens, pos, ast) {
            ast.nodes[node as usize].left = init;
        }
    } else { skip_semi(tokens, pos); }
    // Condition
    if *pos < tokens.len() && tokens[*pos].token_type != TokenType::Semicolon {
        if let Some(cond) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].right = cond;
        }
    }
    skip_semi(tokens, pos);
    // Update
    if *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
        if let Some(update) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].extra = update;
        }
    }
    expect(tokens, pos, TokenType::RightParen);

    if let Some(body) = parse_statement(tokens, pos, ast) {
        ast.nodes[node as usize].body = body;
    }
    Some(node)
}

fn parse_return(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1;
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ReturnStatement;

    if *pos < tokens.len() && tokens[*pos].token_type != TokenType::Semicolon
        && tokens[*pos].token_type != TokenType::RightBrace
    {
        if let Some(expr) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].left = expr;
        }
    }
    skip_semi(tokens, pos);
    Some(node)
}

fn parse_block(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    expect(tokens, pos, TokenType::LeftBrace);
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::BlockStatement;

    let mut last: u16 = 0xFFFF;
    while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightBrace {
        if let Some(stmt) = parse_statement(tokens, pos, ast) {
            if last == 0xFFFF {
                ast.nodes[node as usize].left = stmt;
            } else {
                ast.nodes[last as usize].next = stmt;
            }
            last = stmt;
        } else {
            *pos += 1;
        }
    }
    expect(tokens, pos, TokenType::RightBrace);
    Some(node)
}

/// Parse expression with precedence climbing
fn parse_expression(tokens: &[Token], pos: &mut usize, ast: &mut Ast, min_prec: u8) -> Option<u16> {
    let mut left = parse_unary(tokens, pos, ast)?;

    while *pos < tokens.len() {
        let (op, prec) = get_binary_op(&tokens[*pos]);
        if prec == 0 || prec < min_prec { break; }

        *pos += 1;

        if op == Operator::Assign || op == Operator::PlusAssign
            || op == Operator::MinusAssign || op == Operator::StarAssign
        {
            // Assignment (right-associative)
            let right = parse_expression(tokens, pos, ast, prec)?;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::AssignExpr;
            ast.nodes[node as usize].op = op;
            ast.nodes[node as usize].left = left;
            ast.nodes[node as usize].right = right;
            left = node;
        } else {
            let right = parse_expression(tokens, pos, ast, prec + 1)?;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::BinaryExpr;
            ast.nodes[node as usize].op = op;
            ast.nodes[node as usize].left = left;
            ast.nodes[node as usize].right = right;
            left = node;
        }
    }

    Some(left)
}

fn parse_unary(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    if *pos >= tokens.len() { return None; }

    match tokens[*pos].token_type {
        TokenType::Not | TokenType::Minus | TokenType::Typeof | TokenType::BitNot => {
            let op = match tokens[*pos].token_type {
                TokenType::Not => Operator::Not,
                TokenType::Minus => Operator::Sub,
                TokenType::Typeof => Operator::Typeof,
                TokenType::BitNot => Operator::BitNot,
                _ => Operator::None,
            };
            *pos += 1;
            let operand = parse_unary(tokens, pos, ast)?;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::UnaryExpr;
            ast.nodes[node as usize].op = op;
            ast.nodes[node as usize].left = operand;
            Some(node)
        }
        _ => parse_postfix(tokens, pos, ast),
    }
}

fn parse_postfix(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    let mut expr = parse_primary(tokens, pos, ast)?;

    loop {
        if *pos >= tokens.len() { break; }
        match tokens[*pos].token_type {
            TokenType::LeftParen => {
                // Function call
                *pos += 1;
                let call = ast.alloc()?;
                ast.nodes[call as usize].kind = NodeKind::CallExpr;
                ast.nodes[call as usize].left = expr;

                // Parse arguments
                let mut last_arg: u16 = 0xFFFF;
                while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
                    if let Some(arg) = parse_expression(tokens, pos, ast, 0) {
                        if last_arg == 0xFFFF {
                            ast.nodes[call as usize].extra = arg;
                        } else {
                            ast.nodes[last_arg as usize].next = arg;
                        }
                        last_arg = arg;
                    }
                    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
                }
                expect(tokens, pos, TokenType::RightParen);
                expr = call;
            }
            TokenType::Dot => {
                *pos += 1;
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
                    let member = ast.alloc()?;
                    ast.nodes[member as usize].kind = NodeKind::MemberExpr;
                    ast.nodes[member as usize].left = expr;
                    ast.nodes[member as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    *pos += 1;
                    expr = member;
                }
            }
            TokenType::LeftBracket => {
                *pos += 1;
                let idx_expr = parse_expression(tokens, pos, ast, 0)?;
                expect(tokens, pos, TokenType::RightBracket);
                let member = ast.alloc()?;
                ast.nodes[member as usize].kind = NodeKind::MemberExpr;
                ast.nodes[member as usize].left = expr;
                ast.nodes[member as usize].right = idx_expr;
                expr = member;
            }
            _ => break,
        }
    }

    Some(expr)
}

fn parse_primary(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    if *pos >= tokens.len() { return None; }
    let tok = &tokens[*pos];

    match tok.token_type {
        TokenType::Number => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::NumberLiteral;
            ast.nodes[node as usize].num_value = tok.num_value;
            *pos += 1;
            Some(node)
        }
        TokenType::String => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::StringLiteral;
            ast.nodes[node as usize].set_name(&tok.text[..tok.text_len]);
            *pos += 1;
            Some(node)
        }
        TokenType::Bool => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::BoolLiteral;
            ast.nodes[node as usize].bool_value = &tok.text[..tok.text_len] == b"true";
            *pos += 1;
            Some(node)
        }
        TokenType::Null => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::NullLiteral;
            *pos += 1;
            Some(node)
        }
        TokenType::Identifier => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::Identifier;
            ast.nodes[node as usize].set_name(&tok.text[..tok.text_len]);
            *pos += 1;
            Some(node)
        }
        TokenType::This => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::Identifier;
            ast.nodes[node as usize].set_name(b"this");
            *pos += 1;
            Some(node)
        }
        TokenType::LeftParen => {
            *pos += 1;
            let expr = parse_expression(tokens, pos, ast, 0)?;
            expect(tokens, pos, TokenType::RightParen);
            Some(expr)
        }
        TokenType::Function => parse_function_decl(tokens, pos, ast),
        _ => {
            *pos += 1;
            None
        }
    }
}

fn get_binary_op(tok: &Token) -> (Operator, u8) {
    match tok.token_type {
        TokenType::Assign => (Operator::Assign, 1),
        TokenType::PlusAssign => (Operator::PlusAssign, 1),
        TokenType::MinusAssign => (Operator::MinusAssign, 1),
        TokenType::Or => (Operator::Or, 2),
        TokenType::And => (Operator::And, 3),
        TokenType::BitOr => (Operator::BitOr, 4),
        TokenType::BitXor => (Operator::BitXor, 5),
        TokenType::BitAnd => (Operator::BitAnd, 6),
        TokenType::Equal => (Operator::Equal, 7),
        TokenType::StrictEqual => (Operator::StrictEqual, 7),
        TokenType::NotEqual => (Operator::NotEqual, 7),
        TokenType::StrictNotEqual => (Operator::StrictNotEqual, 7),
        TokenType::Less => (Operator::Less, 8),
        TokenType::Greater => (Operator::Greater, 8),
        TokenType::LessEqual => (Operator::LessEqual, 8),
        TokenType::GreaterEqual => (Operator::GreaterEqual, 8),
        TokenType::In => (Operator::In, 8),
        TokenType::ShiftLeft => (Operator::ShiftLeft, 9),
        TokenType::ShiftRight => (Operator::ShiftRight, 9),
        TokenType::Plus => (Operator::Add, 10),
        TokenType::Minus => (Operator::Sub, 10),
        TokenType::Star => (Operator::Mul, 11),
        TokenType::Slash => (Operator::Div, 11),
        TokenType::Percent => (Operator::Mod, 11),
        _ => (Operator::None, 0),
    }
}

fn expect(tokens: &[Token], pos: &mut usize, expected: TokenType) {
    if *pos < tokens.len() && tokens[*pos].token_type == expected {
        *pos += 1;
    }
}

fn skip_semi(tokens: &[Token], pos: &mut usize) {
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Semicolon {
        *pos += 1;
    }
}
