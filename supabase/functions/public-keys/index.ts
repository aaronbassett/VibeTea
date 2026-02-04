/**
 * Supabase Edge Function: public-keys
 *
 * Returns all monitor public keys from the database.
 * This endpoint is called by the VibeTea server every 30 seconds
 * to refresh its cache of valid monitor public keys.
 *
 * Response format:
 * {
 *   "keys": [
 *     { "source_id": "monitor-1", "public_key": "base64-encoded-key" },
 *     ...
 *   ]
 * }
 *
 * No authentication required (FR-015).
 */

import { createClient } from 'https://esm.sh/@supabase/supabase-js@2.94.0';

// CORS headers for cross-origin requests
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Headers': 'authorization, x-client-info, apikey, content-type',
};

interface PublicKey {
  source_id: string;
  public_key: string;
}

interface PublicKeysResponse {
  keys: PublicKey[];
}

interface ErrorResponse {
  error: string;
}

Deno.serve(async (req: Request): Promise<Response> => {
  // Handle CORS preflight requests
  if (req.method === 'OPTIONS') {
    return new Response('ok', { headers: corsHeaders });
  }

  // Only allow GET requests
  if (req.method !== 'GET') {
    const errorResponse: ErrorResponse = { error: 'Method not allowed' };
    return new Response(JSON.stringify(errorResponse), {
      status: 405,
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }

  try {
    // Create Supabase client using service role for database access
    // Note: We use the anon key since the table has SELECT granted to anon role
    const supabaseUrl = Deno.env.get('SUPABASE_URL');
    const supabaseAnonKey = Deno.env.get('SUPABASE_ANON_KEY');

    if (!supabaseUrl || !supabaseAnonKey) {
      console.error('Missing required environment variables');
      const errorResponse: ErrorResponse = { error: 'Server configuration error' };
      return new Response(JSON.stringify(errorResponse), {
        status: 500,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });
    }

    const supabase = createClient(supabaseUrl, supabaseAnonKey);

    // Fetch all public keys from the database
    const { data, error } = await supabase
      .from('monitor_public_keys')
      .select('source_id, public_key')
      .order('source_id');

    if (error) {
      console.error('Database error:', error.message);
      const errorResponse: ErrorResponse = { error: 'Failed to fetch public keys' };
      return new Response(JSON.stringify(errorResponse), {
        status: 500,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });
    }

    // Return keys in the expected format (FR-016)
    const response: PublicKeysResponse = {
      keys: (data as PublicKey[]) || [],
    };

    return new Response(JSON.stringify(response), {
      status: 200,
      headers: {
        ...corsHeaders,
        'Content-Type': 'application/json',
        // Cache for 10 seconds to reduce database load while still being responsive
        'Cache-Control': 'public, max-age=10',
      },
    });
  } catch (err) {
    console.error('Unexpected error:', err);
    const errorResponse: ErrorResponse = { error: 'Internal server error' };
    return new Response(JSON.stringify(errorResponse), {
      status: 500,
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }
});
