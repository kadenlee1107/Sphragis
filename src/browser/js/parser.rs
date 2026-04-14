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
        TokenType::Throw => parse_throw(tokens, pos, ast),
        TokenType::Try => parse_try(tokens, pos, ast),
        TokenType::Switch => parse_switch(tokens, pos, ast),
        TokenType::Class => parse_class(tokens, pos, ast),
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
    *pos += 1; // skip 'for'

    expect(tokens, pos, TokenType::LeftParen);

    // Check for for...of or for...in
    // Pattern: for (var/let/const ident of/in expr)
    let saved_pos = *pos;
    let mut is_for_of = false;
    let mut is_for_in = false;

    // Try to detect for...of / for...in
    if *pos < tokens.len() && (tokens[*pos].token_type == TokenType::Var
        || tokens[*pos].token_type == TokenType::Let
        || tokens[*pos].token_type == TokenType::Const)
    {
        let decl_pos = *pos + 1;
        if decl_pos < tokens.len() && tokens[decl_pos].token_type == TokenType::Identifier {
            let after_ident = decl_pos + 1;
            if after_ident < tokens.len() {
                if tokens[after_ident].token_type == TokenType::Of {
                    is_for_of = true;
                } else if tokens[after_ident].token_type == TokenType::In {
                    is_for_in = true;
                }
            }
        }
    }

    if is_for_of || is_for_in {
        let node = ast.alloc()?;
        ast.nodes[node as usize].kind = if is_for_of { NodeKind::ForOfStatement } else { NodeKind::ForStatement };
        // Skip var/let/const
        *pos += 1;
        // Parse variable name
        let var_node = ast.alloc()?;
        ast.nodes[var_node as usize].kind = NodeKind::VarDecl;
        if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
            ast.nodes[var_node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
            *pos += 1;
        }
        ast.nodes[node as usize].left = var_node;

        // Skip 'of' or 'in'
        *pos += 1;

        // Parse iterable expression
        if let Some(iterable) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].right = iterable;
        }
        expect(tokens, pos, TokenType::RightParen);

        // Parse body
        if let Some(body) = parse_statement(tokens, pos, ast) {
            ast.nodes[node as usize].body = body;
        }
        return Some(node);
    }

    // Standard for loop
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ForStatement;

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
        // Ternary conditional: a ? b : c
        if tokens[*pos].token_type == TokenType::Question {
            *pos += 1;
            let then_expr = parse_expression(tokens, pos, ast, 0)?;
            expect(tokens, pos, TokenType::Colon);
            let else_expr = parse_expression(tokens, pos, ast, 0)?;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::ConditionalExpr;
            ast.nodes[node as usize].left = left;
            ast.nodes[node as usize].right = then_expr;
            ast.nodes[node as usize].extra = else_expr;
            left = node;
            continue;
        }

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

                // Parse arguments — linked via right → next chain
                let mut last_arg: u16 = 0xFFFF;
                while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
                    if let Some(arg) = parse_expression(tokens, pos, ast, 0) {
                        if last_arg == 0xFFFF {
                            ast.nodes[call as usize].right = arg;
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
                    let prop_node = ast.alloc()?;
                    ast.nodes[prop_node as usize].kind = NodeKind::Identifier;
                    ast.nodes[prop_node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    let member = ast.alloc()?;
                    ast.nodes[member as usize].kind = NodeKind::MemberExpr;
                    ast.nodes[member as usize].left = expr;
                    ast.nodes[member as usize].right = prop_node;
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
            TokenType::OptionalChain => {
                // obj?.prop — desugar to: obj == null ? undefined : obj.prop
                *pos += 1;
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
                    let prop_node = ast.alloc()?;
                    ast.nodes[prop_node as usize].kind = NodeKind::Identifier;
                    ast.nodes[prop_node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    *pos += 1;

                    // Build: expr == null ? undefined : expr.prop
                    // For simplicity, we emit as ConditionalExpr
                    let null_check = ast.alloc()?;
                    ast.nodes[null_check as usize].kind = NodeKind::BinaryExpr;
                    ast.nodes[null_check as usize].op = Operator::Equal;
                    ast.nodes[null_check as usize].left = expr;
                    let null_node = ast.alloc()?;
                    ast.nodes[null_node as usize].kind = NodeKind::NullLiteral;
                    ast.nodes[null_check as usize].right = null_node;

                    let member = ast.alloc()?;
                    ast.nodes[member as usize].kind = NodeKind::MemberExpr;
                    ast.nodes[member as usize].left = expr;
                    ast.nodes[member as usize].right = prop_node;

                    let undef = ast.alloc()?;
                    ast.nodes[undef as usize].kind = NodeKind::Identifier;
                    ast.nodes[undef as usize].set_name(b"undefined");

                    let cond = ast.alloc()?;
                    ast.nodes[cond as usize].kind = NodeKind::ConditionalExpr;
                    ast.nodes[cond as usize].left = null_check;
                    ast.nodes[cond as usize].right = undef; // if null -> undefined
                    ast.nodes[cond as usize].extra = member; // else -> member access

                    expr = cond;
                }
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
            // Check for arrow function: x => body
            if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Arrow {
                *pos += 1; // skip =>
                let arrow = ast.alloc()?;
                ast.nodes[arrow as usize].kind = NodeKind::ArrowFunc;
                // Single parameter
                ast.nodes[arrow as usize].params[0] = node;
                ast.nodes[arrow as usize].param_count = 1;
                // Parse body
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::LeftBrace {
                    if let Some(body) = parse_block(tokens, pos, ast) {
                        ast.nodes[arrow as usize].body = body;
                    }
                } else {
                    if let Some(body) = parse_expression(tokens, pos, ast, 0) {
                        ast.nodes[arrow as usize].body = body;
                    }
                }
                return Some(arrow);
            }
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
            // Could be (expr) or (params) => body (arrow function)
            // Look ahead to detect arrow function
            if is_arrow_function(tokens, *pos) {
                return parse_arrow_function(tokens, pos, ast);
            }
            *pos += 1;
            let expr = parse_expression(tokens, pos, ast, 0)?;
            expect(tokens, pos, TokenType::RightParen);
            Some(expr)
        }
        TokenType::Function => {
            // Function expression (anonymous or named)
            parse_function_decl(tokens, pos, ast)
        }
        TokenType::LeftBracket => {
            // Array literal [el0, el1, ...]
            *pos += 1;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::ArrayLiteral;
            let mut last_elem: u16 = 0xFFFF;
            while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightBracket {
                if let Some(elem) = parse_expression(tokens, pos, ast, 0) {
                    if last_elem == 0xFFFF {
                        ast.nodes[node as usize].left = elem;
                    } else {
                        ast.nodes[last_elem as usize].next = elem;
                    }
                    last_elem = elem;
                }
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
            }
            expect(tokens, pos, TokenType::RightBracket);
            Some(node)
        }
        TokenType::LeftBrace => {
            // Object literal { key: value, ... }
            // Note: this is in expression position, so it's an object, not a block
            *pos += 1;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::ObjectLiteral;
            let mut last_prop: u16 = 0xFFFF;
            while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightBrace {
                // Parse key: value
                if *pos < tokens.len() && (tokens[*pos].token_type == TokenType::Identifier
                    || tokens[*pos].token_type == TokenType::String
                    || tokens[*pos].token_type == TokenType::Number) {
                    let prop = ast.alloc()?;
                    ast.nodes[prop as usize].kind = NodeKind::Identifier; // property entry
                    ast.nodes[prop as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    *pos += 1;

                    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Colon {
                        *pos += 1; // skip ':'
                        if let Some(val) = parse_expression(tokens, pos, ast, 0) {
                            ast.nodes[prop as usize].left = val;
                        }
                    } else {
                        // Shorthand: { foo } means { foo: foo }
                        let ident = ast.alloc()?;
                        ast.nodes[ident as usize].kind = NodeKind::Identifier;
                        let nlen = ast.nodes[prop as usize].name_len;
                        let mut nbuf = [0u8; MAX_IDENT];
                        nbuf[..nlen].copy_from_slice(&ast.nodes[prop as usize].name[..nlen]);
                        ast.nodes[ident as usize].set_name(&nbuf[..nlen]);
                        ast.nodes[prop as usize].left = ident;
                    }

                    if last_prop == 0xFFFF {
                        ast.nodes[node as usize].left = prop;
                    } else {
                        ast.nodes[last_prop as usize].next = prop;
                    }
                    last_prop = prop;
                } else {
                    *pos += 1; // skip unknown
                }
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
            }
            expect(tokens, pos, TokenType::RightBrace);
            Some(node)
        }
        TokenType::New => {
            *pos += 1;
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::NewExpr;
            // Parse the constructor expression
            if let Some(callee) = parse_primary(tokens, pos, ast) {
                ast.nodes[node as usize].left = callee;
            }
            // Parse arguments if present
            if *pos < tokens.len() && tokens[*pos].token_type == TokenType::LeftParen {
                *pos += 1;
                let mut last_arg: u16 = 0xFFFF;
                while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
                    if let Some(arg) = parse_expression(tokens, pos, ast, 0) {
                        if last_arg == 0xFFFF {
                            ast.nodes[node as usize].right = arg;
                        } else {
                            ast.nodes[last_arg as usize].next = arg;
                        }
                        last_arg = arg;
                    }
                    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
                }
                expect(tokens, pos, TokenType::RightParen);
            }
            Some(node)
        }
        TokenType::TemplateNoSub => {
            // Simple template literal with no substitutions — treat as string
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::StringLiteral;
            ast.nodes[node as usize].set_name(&tok.text[..tok.text_len]);
            *pos += 1;
            Some(node)
        }
        TokenType::TemplateStart => {
            // Template literal with substitutions: `text${expr}text...`
            // Build as concatenation of string parts and expressions
            let str_node = ast.alloc()?;
            ast.nodes[str_node as usize].kind = NodeKind::StringLiteral;
            ast.nodes[str_node as usize].set_name(&tok.text[..tok.text_len]);
            *pos += 1;

            let mut result = str_node;

            // Parse expression
            if let Some(expr) = parse_expression(tokens, pos, ast, 0) {
                // Concatenate string + expr
                let concat = ast.alloc()?;
                ast.nodes[concat as usize].kind = NodeKind::BinaryExpr;
                ast.nodes[concat as usize].op = Operator::Add;
                ast.nodes[concat as usize].left = result;
                ast.nodes[concat as usize].right = expr;
                result = concat;
            }

            // Handle TemplateMid and TemplateEnd
            while *pos < tokens.len() {
                let tt = tokens[*pos].token_type;
                if tt == TokenType::TemplateEnd {
                    let end_str = ast.alloc()?;
                    ast.nodes[end_str as usize].kind = NodeKind::StringLiteral;
                    ast.nodes[end_str as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    *pos += 1;
                    if tokens[*pos - 1].text_len > 0 {
                        let concat = ast.alloc()?;
                        ast.nodes[concat as usize].kind = NodeKind::BinaryExpr;
                        ast.nodes[concat as usize].op = Operator::Add;
                        ast.nodes[concat as usize].left = result;
                        ast.nodes[concat as usize].right = end_str;
                        result = concat;
                    }
                    break;
                } else if tt == TokenType::TemplateMid {
                    let mid_str = ast.alloc()?;
                    ast.nodes[mid_str as usize].kind = NodeKind::StringLiteral;
                    ast.nodes[mid_str as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    *pos += 1;
                    if tokens[*pos - 1].text_len > 0 {
                        let concat = ast.alloc()?;
                        ast.nodes[concat as usize].kind = NodeKind::BinaryExpr;
                        ast.nodes[concat as usize].op = Operator::Add;
                        ast.nodes[concat as usize].left = result;
                        ast.nodes[concat as usize].right = mid_str;
                        result = concat;
                    }
                    // Parse next expression
                    if let Some(expr) = parse_expression(tokens, pos, ast, 0) {
                        let concat = ast.alloc()?;
                        ast.nodes[concat as usize].kind = NodeKind::BinaryExpr;
                        ast.nodes[concat as usize].op = Operator::Add;
                        ast.nodes[concat as usize].left = result;
                        ast.nodes[concat as usize].right = expr;
                        result = concat;
                    }
                } else {
                    break;
                }
            }
            Some(result)
        }
        TokenType::Undefined => {
            let node = ast.alloc()?;
            ast.nodes[node as usize].kind = NodeKind::Identifier;
            ast.nodes[node as usize].set_name(b"undefined");
            *pos += 1;
            Some(node)
        }
        _ => {
            *pos += 1;
            None
        }
    }
}

/// Detect if (tokens starting at pos) is an arrow function: (params) => ...
fn is_arrow_function(tokens: &[Token], pos: usize) -> bool {
    // Must start with (
    if pos >= tokens.len() || tokens[pos].token_type != TokenType::LeftParen {
        return false;
    }
    // Find matching )
    let mut depth = 1;
    let mut i = pos + 1;
    while i < tokens.len() && depth > 0 {
        match tokens[i].token_type {
            TokenType::LeftParen => depth += 1,
            TokenType::RightParen => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    // Check if => follows
    i < tokens.len() && tokens[i].token_type == TokenType::Arrow
}

/// Parse arrow function: (params) => body  or  () => body
fn parse_arrow_function(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ArrowFunc;

    // Parse parameters
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

    // Skip =>
    expect(tokens, pos, TokenType::Arrow);

    // Parse body
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::LeftBrace {
        if let Some(body) = parse_block(tokens, pos, ast) {
            ast.nodes[node as usize].body = body;
        }
    } else {
        if let Some(body) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].body = body;
        }
    }

    Some(node)
}

fn parse_throw(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'throw'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ThrowStatement;
    if let Some(expr) = parse_expression(tokens, pos, ast, 0) {
        ast.nodes[node as usize].left = expr;
    }
    skip_semi(tokens, pos);
    Some(node)
}

fn parse_try(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'try'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::TryStatement;

    // Try body
    if let Some(body) = parse_block(tokens, pos, ast) {
        ast.nodes[node as usize].left = body;
    }

    // Catch clause
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Catch {
        *pos += 1;
        let catch_node = ast.alloc()?;
        // Parse catch parameter
        if *pos < tokens.len() && tokens[*pos].token_type == TokenType::LeftParen {
            *pos += 1;
            if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
                ast.nodes[catch_node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                *pos += 1;
            }
            expect(tokens, pos, TokenType::RightParen);
        }
        // Catch body
        if let Some(catch_body) = parse_block(tokens, pos, ast) {
            ast.nodes[catch_node as usize].left = catch_body;
        }
        ast.nodes[node as usize].right = catch_node;
    }

    // Finally clause
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Finally {
        *pos += 1;
        if let Some(finally_body) = parse_block(tokens, pos, ast) {
            ast.nodes[node as usize].extra = finally_body;
        }
    }

    Some(node)
}

fn parse_switch(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'switch'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::SwitchStatement;

    expect(tokens, pos, TokenType::LeftParen);
    if let Some(disc) = parse_expression(tokens, pos, ast, 0) {
        ast.nodes[node as usize].left = disc;
    }
    expect(tokens, pos, TokenType::RightParen);
    expect(tokens, pos, TokenType::LeftBrace);

    // Parse case clauses — linked via next chain
    let mut last_case: u16 = 0xFFFF;
    while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightBrace {
        if tokens[*pos].token_type == TokenType::Case || tokens[*pos].token_type == TokenType::Default {
            let is_default = tokens[*pos].token_type == TokenType::Default;
            *pos += 1;
            let case_node = ast.alloc()?;
            ast.nodes[case_node as usize].kind = NodeKind::ExprStatement; // reuse for case

            if !is_default {
                if let Some(test) = parse_expression(tokens, pos, ast, 0) {
                    ast.nodes[case_node as usize].left = test;
                }
            }
            expect(tokens, pos, TokenType::Colon);

            // Parse case body statements
            let mut last_stmt: u16 = 0xFFFF;
            while *pos < tokens.len()
                && tokens[*pos].token_type != TokenType::Case
                && tokens[*pos].token_type != TokenType::Default
                && tokens[*pos].token_type != TokenType::RightBrace
            {
                if let Some(stmt) = parse_statement(tokens, pos, ast) {
                    if last_stmt == 0xFFFF {
                        ast.nodes[case_node as usize].right = stmt;
                    } else {
                        ast.nodes[last_stmt as usize].next = stmt;
                    }
                    last_stmt = stmt;
                } else {
                    *pos += 1;
                }
            }

            if last_case == 0xFFFF {
                ast.nodes[node as usize].right = case_node;
            } else {
                ast.nodes[last_case as usize].next = case_node;
            }
            last_case = case_node;
        } else {
            *pos += 1;
        }
    }
    expect(tokens, pos, TokenType::RightBrace);
    Some(node)
}

/// Parse class declaration — desugar to function + prototype
fn parse_class(tokens: &[Token], pos: &mut usize, ast: &mut Ast) -> Option<u16> {
    *pos += 1; // skip 'class'
    let node = ast.alloc()?;
    ast.nodes[node as usize].kind = NodeKind::ClassDecl;

    // Class name
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
        ast.nodes[node as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
        *pos += 1;
    }

    // Optional extends
    if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Extends {
        *pos += 1;
        if let Some(parent) = parse_expression(tokens, pos, ast, 0) {
            ast.nodes[node as usize].extra = parent;
        }
    }

    // Class body { ... }
    expect(tokens, pos, TokenType::LeftBrace);
    let mut last_method: u16 = 0xFFFF;

    while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightBrace {
        // Parse method: name(...) { ... }
        if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Identifier {
            let method = ast.alloc()?;
            ast.nodes[method as usize].kind = NodeKind::FunctionDecl;
            ast.nodes[method as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
            *pos += 1;

            // Parameters
            expect(tokens, pos, TokenType::LeftParen);
            let mut pcount = 0u8;
            while *pos < tokens.len() && tokens[*pos].token_type != TokenType::RightParen {
                if tokens[*pos].token_type == TokenType::Identifier {
                    let param = ast.alloc()?;
                    ast.nodes[param as usize].kind = NodeKind::Identifier;
                    ast.nodes[param as usize].set_name(&tokens[*pos].text[..tokens[*pos].text_len]);
                    if (pcount as usize) < 8 {
                        ast.nodes[method as usize].params[pcount as usize] = param;
                        pcount += 1;
                    }
                    *pos += 1;
                }
                if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Comma { *pos += 1; }
            }
            ast.nodes[method as usize].param_count = pcount;
            expect(tokens, pos, TokenType::RightParen);

            // Body
            if let Some(body) = parse_block(tokens, pos, ast) {
                ast.nodes[method as usize].body = body;
            }

            if last_method == 0xFFFF {
                ast.nodes[node as usize].left = method;
            } else {
                ast.nodes[last_method as usize].next = method;
            }
            last_method = method;
        } else {
            *pos += 1; // skip unknown
        }
        // Skip optional semicolons between methods
        if *pos < tokens.len() && tokens[*pos].token_type == TokenType::Semicolon {
            *pos += 1;
        }
    }
    expect(tokens, pos, TokenType::RightBrace);

    Some(node)
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
